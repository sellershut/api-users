use api_core::{api::MutateVerificationToken, VerificationToken};
use api_database::Client;
use async_graphql::{Context, Object};
use tracing::instrument;

#[derive(Default, Debug)]
pub struct VerificationTokenMutation;

#[Object]
impl VerificationTokenMutation {
    #[instrument(skip(ctx), err(Debug))]
    async fn create_verification_token(
        &self,
        ctx: &Context<'_>,
        input: VerificationToken,
    ) -> async_graphql::Result<VerificationToken> {
        let database = ctx.data::<Client>()?;

        let verification_token = database.create_verification_token(&input).await?;
        Ok(verification_token)
    }

    #[instrument(skip(ctx), err(Debug))]
    async fn delete_verification_token(
        &self,
        ctx: &Context<'_>,
        token: String,
        identifier: String,
    ) -> async_graphql::Result<Option<VerificationToken>> {
        let database = ctx.data::<Client>()?;

        Ok(database.use_verification_token(identifier, token).await?)
    }
}
