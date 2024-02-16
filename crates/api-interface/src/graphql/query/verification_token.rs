use api_core::{api::QueryVerificationToken, VerificationToken};
use async_graphql::{Context, Object};
use tracing::instrument;

use crate::graphql::extract_db;

#[derive(Default, Debug)]
pub struct VerificationTokenQuery;

#[Object]
impl VerificationTokenQuery {
    #[instrument(skip(ctx), err(Debug))]
    async fn verification_token(
        &self,
        ctx: &Context<'_>,
        token: String,
        identifier: String,
    ) -> async_graphql::Result<Option<VerificationToken>> {
        let database = extract_db(ctx)?;

        let verification_token = database.get_verification_token(token, identifier).await?;
        Ok(verification_token)
    }
}
