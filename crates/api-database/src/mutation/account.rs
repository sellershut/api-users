use api_core::{
    api::{CoreError, MutateAccounts},
    Account,
};
use std::fmt::Debug;
use surrealdb::opt::RecordId;
use uuid::Uuid;

use crate::{collections::Collections, entity::DatabaseEntityAccount, map_db_error, Client};

impl MutateAccounts for Client {
    async fn link_account(&self, account: &Account) -> Result<Account, CoreError> {
        let input_account = InputAccount::from(account);

        let id = Uuid::now_v7().to_string();
        let item: Option<DatabaseEntityAccount> = self
            .client
            .create((Collections::Account.to_string(), id))
            .content(input_account)
            .await
            .map_err(map_db_error)?;

        match item {
            Some(e) => Account::try_from(e),
            None => Err(CoreError::Unreachable),
        }
    }

    async fn unlink_account(
        &self,
        provider: impl AsRef<str> + Send + Debug,
        provider_account_id: impl AsRef<str> + Send + Debug,
    ) -> Result<(), CoreError> {
        let provider = provider.as_ref();
        let provider_account_id = provider_account_id.as_ref();

        self.client
            .query(format!(
            "DELETE {} WHERE providerAccountId = {provider_account_id} AND provider = {provider}",
            Collections::Account
        ))
            .await
            .map_err(map_db_error)?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
struct InputAccount<'a> {
    user: RecordId,
    #[serde(rename = "type")]
    account_type: &'a str,
    provider: &'a str,
    provider_account_id: &'a str,
    refresh_token: Option<&'a str>,
    access_token: Option<&'a str>,
    expires_at: Option<usize>,
    id_token: &'a str,
    scope: &'a str,
    session_state: &'a str,
    token_type: &'a str,
}

impl<'a> From<&'a Account> for InputAccount<'a> {
    fn from(value: &'a Account) -> Self {
        Self {
            user: RecordId::from(("account", value.user.to_string().as_str())),
            account_type: &value.account_type,
            provider: &value.provider,
            provider_account_id: &value.provider_account_id,
            refresh_token: value.refresh_token.as_deref(),
            access_token: value.access_token.as_deref(),
            expires_at: value.expires_at,
            id_token: &value.id_token,
            scope: &value.scope,
            session_state: &value.session_state,
            token_type: &value.token_type,
        }
    }
}
