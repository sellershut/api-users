use api_core::{api::LocalMutateSessions, Session};
use api_database::Client;
use async_graphql::{Context, Object};
use time::OffsetDateTime;
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
        id: String,
        expires_at: OffsetDateTime,
    ) -> async_graphql::Result<Option<Session>> {
        let database = ctx.data::<Client>()?;

        Ok(database.update_session(id, &expires_at).await?)
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

    #[instrument(skip(ctx), err(Debug))]
    async fn delete_expired_sessions(&self, ctx: &Context<'_>) -> async_graphql::Result<String> {
        let database = ctx.data::<Client>()?;

        database.delete_expired_sessions().await?;
        Ok(String::from("expired sessions cleared"))
    }

    #[instrument(skip(ctx), err(Debug))]
    async fn delete_user_session(
        &self,
        ctx: &Context<'_>,
        user_id: String,
    ) -> async_graphql::Result<String> {
        let database = ctx.data::<Client>()?;

        database.delete_user_sessions(user_id).await?;
        Ok(String::from("user sessions cleared"))
    }
}
