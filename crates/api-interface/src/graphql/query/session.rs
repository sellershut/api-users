use api_core::{api::QuerySessions, reexports::uuid::Uuid, Session};
use async_graphql::{Context, Object};
use tracing::instrument;

use crate::graphql::extract_db;

#[derive(Default, Debug)]
pub struct SessionQuery;

#[Object]
impl SessionQuery {
    #[instrument(skip(ctx), err(Debug))]
    async fn user_sessions(
        &self,
        ctx: &Context<'_>,
        user_id: Uuid,
    ) -> async_graphql::Result<Vec<Session>> {
        let database = extract_db(ctx)?;

        let categories = database.get_user_sessions(&user_id).await?;

        Ok(categories.collect())
    }
}
