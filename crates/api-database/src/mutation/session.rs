use api_core::{
    api::{CoreError, MutateSessions},
    Session,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use surrealdb::opt::RecordId;
use time::OffsetDateTime;
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::{collections::Collection, entity::DatabaseEntitySession, map_db_error, Client};

impl MutateSessions for Client {
    #[instrument(skip(self), err(Debug))]
    async fn create_session(&self, session: &Session) -> Result<Session, CoreError> {
        let input_session = InputSession::from(session);
        let id = Uuid::now_v7().to_string();
        let item: Option<DatabaseEntitySession> = self
            .client
            .create((Collection::Session.to_string(), id))
            .content(input_session)
            .await
            .map_err(map_db_error)?;

        let session = match item {
            Some(e) => Session::try_from(e),
            None => Err(CoreError::Unreachable),
        }?;

        debug!("session created");

        Ok(session)
    }

    #[instrument(skip(self), err(Debug))]
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
            .query(
                "SELECT * FROM type::table($table) WHERE session_token = type::string($token) LIMIT 1",
            ).bind(("table", Collection::Session)).bind(("token", session_token))
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

            debug!("session updated");

            Ok(res)
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self), err(Debug))]
    async fn delete_session(&self, id: impl AsRef<str> + Send + Debug) -> Result<(), CoreError> {
        let _resp: Option<DatabaseEntitySession> = self
            .client
            .delete((Collection::Session.to_string(), id.as_ref().to_string()))
            .await
            .map_err(map_db_error)?;

        debug!("session deleted");

        Ok(())
    }

    #[instrument(skip(self), err(Debug))]
    async fn delete_user_sessions(
        &self,
        user_id: impl AsRef<str> + Send + Debug,
    ) -> Result<(), CoreError> {
        let user_id = user_id.as_ref();
        self.client
            .query("DELETE * FROM type::table($table) WHERE user_id = type::string($user_id)")
            .bind(("table", Collection::Session))
            .bind(("user_id", user_id))
            .await
            .map_err(map_db_error)?;

        debug!("user sessions deleted");

        Ok(())
    }

    #[instrument(skip(self), err(Debug))]
    async fn delete_expired_sessions(&self) -> Result<(), CoreError> {
        self.client
            .query("DELETE * FROM type::table($table) WHERE expires_at <= time::now()")
            .bind(("table", Collection::Session))
            .await
            .map_err(map_db_error)?;

        debug!("expired sessions deleted");

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
                Collection::User.to_string().as_str(),
                value.user.to_string().as_str(),
            )),
            expires_at: &value.expires_at,
            session_token: &value.session_token,
        }
    }
}
