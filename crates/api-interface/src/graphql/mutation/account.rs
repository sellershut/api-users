use api_core::{api::LocalMutateAccounts, Account};
use api_database::Client;
use async_graphql::{Context, Object};
use tracing::instrument;

#[derive(Default, Debug)]
pub struct AccountMutation;

#[Object]
impl AccountMutation {
    #[instrument(skip(ctx), err(Debug))]
    async fn create_account(
        &self,
        ctx: &Context<'_>,
        input: Account,
    ) -> async_graphql::Result<Account> {
        let database = ctx.data::<Client>()?;

        let account = database.link_account(&input).await?;
        Ok(account)
    }

    #[instrument(skip(ctx), err(Debug))]
    async fn delete_account(
        &self,
        ctx: &Context<'_>,
        provider_account_id: String,
        provider: String,
    ) -> async_graphql::Result<String> {
        let database = ctx.data::<Client>()?;

        database
            .unlink_account(provider, provider_account_id)
            .await?;
        Ok(String::from("item deleted"))
    }
}
