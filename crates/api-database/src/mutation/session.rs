use api_core::{
    api::{CoreError, MutateSessions},
    Session,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use surrealdb::opt::RecordId;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{collections::Collections, entity::DatabaseEntitySession, map_db_error, Client};

impl MutateSessions for Client {
    async fn create_session(&self, session: &Session) -> Result<Session, CoreError> {
        let input_session = InputSession::from(session);
        let id = Uuid::now_v7().to_string();
        let item: Option<DatabaseEntitySession> = self
            .client
            .create((Collections::Session.to_string(), id))
            .content(input_session)
            .await
            .map_err(map_db_error)?;

        let session = match item {
            Some(e) => Session::try_from(e),
            None => Err(CoreError::Unreachable),
        }?;

        /* let cache_key = CacheKey::Session {
            token: &session.session_token,
        };

        if let Some((ref redis_pool, _)) = self.redis {
            if let Err(e) = redis_query::update(cache_key, redis_pool, &session, 86400000).await {
                error!(key = %cache_key, "[redis update]: {e}");
            }
        } */

        Ok(session)
    }

    async fn update_session(
        &self,
        id: impl AsRef<str> + Send + Debug,
        expires_at: &OffsetDateTime,
    ) -> Result<Option<Session>, CoreError> {
        #[derive(Serialize, Deserialize)]
        struct Document {
            expires_at: OffsetDateTime,
        }

        let session_token = id.as_ref();

        let mut resp = self
            .client
            .query(format!(
                "SELECT * FROM {} WHERE session_token = '{session_token}' LIMIT 1",
                Collections::Session
            ))
            .await
            .map_err(map_db_error)?;

        let resp: Option<DatabaseEntitySession> = resp.take(0).map_err(map_db_error)?;

        if let Some(session) = resp {
            let resp: Option<DatabaseEntitySession> = self
                .client
                .update(session.id)
                .merge(Document {
                    expires_at: expires_at.to_owned(),
                })
                .await
                .map_err(map_db_error)?;

            let res = match resp {
                Some(res) => Some(Session::try_from(res)?),
                None => None,
            };
            Ok(res)
        } else {
            Ok(None)
        }
    }

    async fn delete_session(&self, id: impl AsRef<str> + Send + Debug) -> Result<(), CoreError> {
        let _resp: Option<DatabaseEntitySession> = self
            .client
            .delete((Collections::Session.to_string(), id.as_ref().to_string()))
            .await
            .map_err(map_db_error)?;
        Ok(())
    }

    async fn delete_user_sessions(
        &self,
        user_id: impl AsRef<str> + Send + Debug,
    ) -> Result<(), CoreError> {
        let user_id = user_id.as_ref();
        self.client
            .query(format!(
                "DELETE * FROM {} WHERE user_id = {user_id}",
                Collections::Session
            ))
            .await
            .map_err(map_db_error)?;
        Ok(())
    }

    async fn delete_expired_sessions(&self) -> Result<(), CoreError> {
        self.client
            .query(format!(
                "DELETE * FROM {} WHERE expires_at <= time::now()",
                Collections::Session
            ))
            .await
            .map_err(map_db_error)?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
struct InputSession<'a> {
    user: RecordId,
    expires_at: &'a OffsetDateTime,
    session_token: &'a str,
}

impl<'a> From<&'a Session> for InputSession<'a> {
    fn from(value: &'a Session) -> Self {
        Self {
            user: RecordId::from((
                Collections::User.to_string().as_str(),
                value.user.to_string().as_str(),
            )),
            expires_at: &value.expires_at,
            session_token: &value.session_token,
        }
    }
}
