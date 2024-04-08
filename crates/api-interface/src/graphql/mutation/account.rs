use api_core::api::LocalMutateAccounts;
use api_database::Client;
use async_graphql::{Context, Object, SimpleObject};
use tracing::instrument;
use uuid::Uuid;

#[derive(Default, Debug)]
pub struct AccountMutation;

#[derive(SimpleObject)]
struct Account {
    user_id: Uuid,
    provider: String,
    account_id: String,
}

#[Object]
impl AccountMutation {
    #[instrument(skip(ctx), err(Debug))]
    async fn create_account(
        &self,
        ctx: &Context<'_>,
        provider_name: String,
        provider_account_id: String,
        user_id: Uuid,
    ) -> async_graphql::Result<Account> {
        let database = ctx.data::<Client>()?;

        database
            .link_account(&provider_name, &provider_account_id, &user_id)
            .await?;

        Ok(Account {
            provider: provider_name,
            account_id: provider_account_id,
            user_id,
        })
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
