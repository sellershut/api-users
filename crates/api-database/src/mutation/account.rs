use api_core::api::{CoreError, MutateAccounts};
use serde_json::json;
use std::fmt::Debug;
use surrealdb::sql::Thing;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::{collections::Collection, entity::DatabaseEntityAccountProvider, map_db_error, Client};

impl MutateAccounts for Client {
    #[instrument(skip(self), err(Debug))]
    async fn link_account(
        &self,
        provider: impl AsRef<str> + Send + Debug + Sync,
        provider_account_id: impl AsRef<str> + Send + Debug + Sync,
        user_id: &Uuid,
    ) -> Result<(), CoreError> {
        let create_id = |id: &Uuid| -> Thing {
            Thing::from((
                Collection::User.to_string().as_str(),
                id.to_string().as_str(),
            ))
        };

        let provider = provider.as_ref();

        let mut resp = self
            .client
            .query(
                "SELECT out.* FROM type::table($table) WHERE provider_account_id = type::string($provider_account_id) AND out.name = type::string($provider)"
            )
            .bind(("table", Collection::UserAccount))
            .bind(("provider", provider))
            .bind(("provider_account_id", provider_account_id.as_ref()))
            .await
            .map_err(map_db_error)?;

        let account: Vec<DatabaseEntityAccountProvider> = resp.take(0).map_err(map_db_error)?;

        if account.first().is_none() {
            let user_id = create_id(user_id);
            let mut resp =self.client
                .query("SELECT value id FROM type::table($account_provider_table) WHERE name = type::string($name)")
                .bind(("account_provider_table", Collection::AccountProvider)).await.map_err(map_db_error)?;

            let create_account = |id: Thing| {
                let query = format!(
                    "RELATE {user_id} -> {} -> {id} SET provider_account_id = '{}'",
                    Collection::UserAccount,
                    provider_account_id.as_ref()
                );
                println!("{query}");
                self.client.query(query)
            };

            let id: Option<Thing> = resp.take(0).map_err(map_db_error)?;
            if let Some(id) = id {
                let res = create_account(id).await.map_err(map_db_error)?;
                println!("{res:?}");
            } else {
                let id = Uuid::now_v7();
                let account_provider: Option<DatabaseEntityAccountProvider> = self
                    .client
                    .create((Collection::AccountProvider.to_string(), id.to_string()))
                    .content(json!({
                        "name": provider
                    }))
                    .await
                    .map_err(map_db_error)?;

                if let Some(account_provider) = account_provider {
                    let res = create_account(account_provider.id)
                        .await
                        .map_err(map_db_error)?;
                    println!("{res:?}");
                } else {
                    unreachable!("expected account provider to be returned");
                }
            };
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
