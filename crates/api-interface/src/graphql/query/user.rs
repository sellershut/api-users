use api_core::{api::QueryUsers, reexports::uuid::Uuid, Session, User};
use async_graphql::{Context, Object, SimpleObject};
use tracing::instrument;

use crate::graphql::{extract_db, query::Params};

use super::{pagination::paginate, ConnectionResult};

#[derive(Default, Debug)]
pub struct UserQuery;

#[derive(SimpleObject)]
pub struct SearchResult {
    user: User,
    parent_name: Option<String>,
}

#[derive(SimpleObject)]
pub struct SessionAndUser {
    session: Session,
    user: User,
}

#[Object]
impl UserQuery {
    #[instrument(skip(ctx), err(Debug))]
    async fn users(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(min_length = 1, max_length = 100))] after: Option<String>,
        #[graphql(validator(min_length = 1, max_length = 100))] before: Option<String>,
        #[graphql(validator(minimum = 1, maximum = 100))] first: Option<i32>,
        #[graphql(validator(minimum = 1, maximum = 100))] last: Option<i32>,
    ) -> ConnectionResult<User> {
        let p = Params::new(after, before, first, last)?;

        let database = extract_db(ctx)?;

        let categories = database.get_users().await?;

        paginate(categories, p, 100).await
    }

    #[instrument(skip(ctx), err(Debug))]
    async fn user_by_id(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<Option<User>> {
        let database = extract_db(ctx)?;

        database.get_user_by_id(&id).await.map_err(|e| e.into())
    }

    #[instrument(skip(ctx), err(Debug))]
    async fn user_by_email(
        &self,
        ctx: &Context<'_>,
        email: String,
    ) -> async_graphql::Result<Option<User>> {
        let database = extract_db(ctx)?;

        database
            .get_user_by_email(&email)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(ctx), err(Debug))]
    async fn user_by_account(
        &self,
        ctx: &Context<'_>,
        provider_account_id: String,
        provider: String,
    ) -> async_graphql::Result<Option<User>> {
        let database = extract_db(ctx)?;

        database
            .get_user_by_account(&provider, &provider_account_id)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(ctx), err(Debug))]
    async fn user_and_session(
        &self,
        ctx: &Context<'_>,
        session_token: String,
    ) -> async_graphql::Result<Option<SessionAndUser>> {
        let database = extract_db(ctx)?;

        let res = database.get_session_and_user(&session_token).await?;

        let res = res.map(|(user, session)| SessionAndUser { session, user });
        Ok(res)
    }

    #[instrument(skip(ctx), err(Debug))]
    async fn search(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(min_length = 1, max_length = 100))] query: String,
        #[graphql(validator(min_length = 1, max_length = 100))] after: Option<String>,
        #[graphql(validator(min_length = 1, max_length = 100))] before: Option<String>,
        #[graphql(validator(minimum = 1, maximum = 100))] first: Option<i32>,
        #[graphql(validator(minimum = 1, maximum = 100))] last: Option<i32>,
    ) -> ConnectionResult<User> {
        let p = Params::new(after, before, first, last)?;

        let database = extract_db(ctx)?;

        let categories = database.search(&query).await?;

        paginate(categories, p, 100).await
    }
}
