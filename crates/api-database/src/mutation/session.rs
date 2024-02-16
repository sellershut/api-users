use api_core::{
    api::{CoreError, MutateSessions},
    Session,
};
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

        match item {
            Some(e) => Session::try_from(e),
            None => Err(CoreError::Unreachable),
        }
    }

    async fn update_session(
        &self,
        session_token: impl AsRef<str> + Send + Debug,
        session: &Session,
    ) -> Result<Option<Session>, CoreError> {
        let mut item = self
            .client
            .query(format!(
                "SELECT * FROM {} WHERE session_token = {}",
                Collections::Session,
                session_token.as_ref()
            ))
            .await
            .map_err(map_db_error)?;

        let resp: Option<DatabaseEntitySession> = item.take(0).map_err(map_db_error)?;
        if let Some(session_value) = resp {
            let id = &session_value.id;
            let ret_val: Option<DatabaseEntitySession> = self
                .client
                .update(id)
                .content(session)
                .await
                .map_err(map_db_error)?;

            let res = match ret_val {
                Some(e) => Some(Session::try_from(e)?),
                None => None,
            };

            Ok(res)
        } else {
            Err(CoreError::Database("Session does not exist".to_string()))
        }
    }

    async fn delete_session(
        &self,
        session_token: impl AsRef<str> + Send + Debug,
    ) -> Result<(), CoreError> {
        let session_token = session_token.as_ref();
        self.client
            .query(format!(
                "DELETE {} WHERE session_token = {session_token} ",
                Collections::Session
            ))
            .await
            .map_err(map_db_error)?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
struct InputSession<'a> {
    id: RecordId,
    user: RecordId,
    expires: &'a OffsetDateTime,
    session_token: &'a str,
}

impl<'a> From<&'a Session> for InputSession<'a> {
    fn from(value: &'a Session) -> Self {
        Self {
            id: RecordId::from((
                Collections::Session.to_string().as_str(),
                value.id.to_string().as_str(),
            )),
            user: RecordId::from((
                Collections::User.to_string().as_str(),
                value.user.to_string().as_str(),
            )),
            expires: &value.expires,
            session_token: &value.session_token,
        }
    }
}
