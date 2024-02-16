use api_core::{api::LocalMutateSessions, Session};
use api_database::Client;
use async_graphql::{Context, Object};
use tracing::instrument;

#[derive(Default, Debug)]
pub struct SessionMutation;

#[Object]
impl SessionMutation {
    #[instrument(skip(ctx), err(Debug))]
    async fn create_session(
        &self,
        ctx: &Context<'_>,
        input: Session,
    ) -> async_graphql::Result<Session> {
        let database = ctx.data::<Client>()?;

        let account = database.create_session(&input).await?;
        Ok(account)
    }

    #[instrument(skip(ctx), err(Debug))]
    async fn update_session(
        &self,
        ctx: &Context<'_>,
        session_token: String,
        data: Session,
    ) -> async_graphql::Result<Option<Session>> {
        let database = ctx.data::<Client>()?;

        Ok(database.update_session(session_token, &data).await?)
    }

    #[instrument(skip(ctx), err(Debug))]
    async fn delete_session(
        &self,
        ctx: &Context<'_>,
        session_token: String,
    ) -> async_graphql::Result<String> {
        let database = ctx.data::<Client>()?;

        database.delete_session(session_token).await?;
        Ok(String::from("item deleted"))
    }
}
