use api_core::{
    api::{CoreError, MutateAccounts},
    Account,
};
use std::fmt::Debug;
use surrealdb::opt::RecordId;
use tracing::instrument;
use uuid::Uuid;

use crate::{collections::Collection, entity::DatabaseEntityAccount, map_db_error, Client};

impl MutateAccounts for Client {
    #[instrument(skip(self), err(Debug))]
    async fn link_account(&self, account: &Account) -> Result<Account, CoreError> {
        let input_account = InputAccount::from(account);

        let mut resp = self
            .client
            .query(
                "SELECT * FROM type::table($table) where provider = type::string($provider) AND provider_account_id = type::string($provider_account_id)"
            )
            .bind(("table", Collection::Account))
            .bind(("provider", &account.provider))
            .bind(("provider_account_id", &account.provider_account_id))
            .await
            .map_err(map_db_error)?;

        let account: Vec<DatabaseEntityAccount> = resp.take(0).map_err(map_db_error)?;
        if let Some(first) = account.first() {
            let value = first.clone();
            Account::try_from(value)
        } else {
            let id = Uuid::now_v7().to_string();
            let item: Option<DatabaseEntityAccount> = self
                .client
                .create((Collection::Account.to_string(), id))
                .content(input_account)
                .await
                .map_err(map_db_error)?;

            match item {
                Some(e) => Account::try_from(e),
                None => Err(CoreError::Unreachable),
            }
        }
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
            .query("DELETE type::table($table) WHERE providerAccountId = type::string($provider_account_id) AND provider = type::string($provider)").bind(("table", Collection::Account)).bind(("provider", provider)).bind(("provider_account_id", provider_account_id))
            .await
            .map_err(map_db_error)?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
struct InputAccount<'a> {
    user: RecordId,
    provider: &'a str,
    provider_account_id: &'a str,
}

impl<'a> From<&'a Account> for InputAccount<'a> {
    fn from(value: &'a Account) -> Self {
        Self {
            user: RecordId::from((Collection::User.to_string(), value.user.to_string())),
            provider: &value.provider,
            provider_account_id: &value.provider_account_id,
        }
    }
}
