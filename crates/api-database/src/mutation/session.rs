use api_core::{
    api::{CoreError, MutateSessions},
    Session,
};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, str::FromStr};
use surrealdb::{
    opt::RecordId,
    sql::{Datetime, Thing},
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tracing::{debug, instrument, warn};
use uuid::Uuid;

use crate::{collections::Collection, entity::DatabaseEntitySession, map_db_error, Client};

impl MutateSessions for Client {
    #[instrument(skip(self), err(Debug))]
    async fn create_session(&self, session: &Session) -> Result<(), CoreError> {
        let create_id = |id: &Uuid| -> Thing {
            Thing::from((
                Collection::User.to_string().as_str(),
                id.to_string().as_str(),
            ))
        };

        let user_id = create_id(&session.user_id);

        let stmt = Collection::UserSession;

        let mut resp = self
            .client
            .query("SELECT VALUE id FROM type::table($account_provider_table) WHERE name = type::string($provider)")
            .bind(("account_provider_table",Collection::AccountProvider))
            .bind(("provider", &session.account_provider.name)).await.map_err(map_db_error)?;

        let dt = session
            .expires_at
            .format(&Rfc3339)
            .map_err(|e| CoreError::Other(e.to_string()))
            .map(|val| {
                Datetime::from_str(&val).map_err(|_| {
                    CoreError::Other(format!("could not parse date time from string: {val}"))
                })
            })??;

        let id: Option<Thing> = resp.take(0).map_err(map_db_error)?;
        if let Some(id) = id {
            self.client.query(format!("RELATE {user_id} -> {stmt} -> {id} SET session_token = type::string($session_token), expires_at = <datetime>type::datetime($expires);"))
            .bind(("session_token", &session.session_token))
            .bind(("expires", dt)).await.map_err(map_db_error)?;
        } else {
            warn!("session not created");
        };

        Ok(())
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
                "SELECT VALUE id FROM type::table($table) WHERE session_token = type::string($session_token) LIMIT 1",
            ).bind(("table", Collection::UserSession)).bind(("session_token", session_token))
            .await
            .map_err(map_db_error)?;

        let resp: Option<RecordId> = resp.take(0).map_err(map_db_error)?;

        if let Some(session) = resp {
            let resp: Option<DatabaseEntitySession> = self
                .client
                .update(session)
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
            warn!("session not found");
            Ok(None)
        }
    }

    #[instrument(skip(self), err(Debug))]
    async fn delete_session(&self, id: impl AsRef<str> + Send + Debug) -> Result<(), CoreError> {
        self
            .client
            .query("DELETE FROM type::table($table) WHERE session_token = type::string($session_token)")
            .bind(("table", Collection::UserSession))
            .bind(("session_token", id.as_ref()))
            .await
            .map_err(map_db_error)?;

        debug!("session deleted");

        Ok(())
    }

    #[instrument(skip(self), err(Debug))]
    async fn delete_user_sessions(&self, user_id: &Uuid) -> Result<(), CoreError> {
        let create_id = |id: &Uuid| -> Thing {
            Thing::from((
                Collection::User.to_string().as_str(),
                id.to_string().as_str(),
            ))
        };
        let id = create_id(user_id);

        self.client
            .query("DELETE * FROM type::table($table) WHERE in = type::record($user_id)")
            .bind(("table", Collection::UserSession))
            .bind(("user_id", id))
            .await
            .map_err(map_db_error)?;

        debug!("user sessions deleted");

        Ok(())
    }

    #[instrument(skip(self), err(Debug))]
    async fn delete_expired_sessions(&self) -> Result<(), CoreError> {
        self.client
            .query("DELETE * FROM type::table($table) WHERE expires_at <= time::now()")
            .bind(("table", Collection::UserSession))
            .await
            .map_err(map_db_error)?;

        debug!("expired sessions deleted");

        Ok(())
    }
}
