use api_core::api::{CoreError, MutateAccounts};
use std::fmt::Debug;
use surrealdb::sql::Thing;
use tracing::instrument;
use uuid::Uuid;

use crate::{collections::Collection, entity::DatabaseEntityAccountProvider, map_db_error, Client};

impl MutateAccounts for Client {
    #[instrument(skip(self), err(Debug))]
    async fn link_account(
        &self,
        provider: impl AsRef<str> + Send + Debug,
        provider_account_id: impl AsRef<str> + Send + Debug,
        user_id: &Uuid,
    ) -> Result<(), CoreError> {
        let create_id = |id: &Uuid| -> Thing {
            Thing::from((
                Collection::User.to_string().as_str(),
                id.to_string().as_str(),
            ))
        };

        let mut resp = self
            .client
            .query(
                "SELECT out.* FROM type::table($table) WHERE provider_account_id = type::string($provider_account_id) AND out.name = type::string($provider)"
            )
            .bind(("table", Collection::UserAccount))
            .bind(("provider", provider.as_ref()))
            .bind(("provider_account_id", provider_account_id.as_ref()))
            .await
            .map_err(map_db_error)?;

        let account: Vec<DatabaseEntityAccountProvider> = resp.take(0).map_err(map_db_error)?;

        if account.first().is_none() {
            self   .client
                .query("
                    BEGIN TRANSACTION;
                    LET $account_provider_id = rand::uuid::v7();
                    LET $acc_prov = CREATE account_provider SET id = $account_provider_id, name = type::string($name);
                    RELATE type::record($user_id) -> type::string($edge_collection) -> $acc_prov.id SET provider_account_id = type::string($provider_account_id);
                    COMMIT TRANSACTION;
                    ")
                .bind(("name", provider.as_ref()))
                .bind(("edge_collection", Collection::UserAccount))
                .bind(("user_id", create_id(user_id)))
                .bind(("provider_account_id", provider_account_id.as_ref()))
                .await
                .map_err(map_db_error)?;
        }
        Ok(())
    }

    #[instrument(skip(self), err(Debug))]
    async fn unlink_account(
        &self,
        provider: impl AsRef<str> + Send + Debug,
        provider_account_id: impl AsRef<str> + Send + Debug,
    ) -> Result<(), CoreError> {
        let provider = provider.as_ref();
        let provider_account_id = provider_account_id.as_ref();

        self.client
            .query("DELETE type::table($table) WHERE provider_account_id = type::string($provider_account_id) AND out.name = type::string($provider)")
            .bind(("table", Collection::UserAccount))
            .bind(("provider", provider))
            .bind(("provider_account_id", provider_account_id))
            .await
            .map_err(map_db_error)?;

        Ok(())
    }
}
