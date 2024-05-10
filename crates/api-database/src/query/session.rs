use api_core::{
    api::{CoreError, QuerySessions},
    Session,
};
use surrealdb::sql::Thing;
use tracing::{event, trace, Level};
use uuid::Uuid;

use crate::{collections::Collection, entity::DatabaseEntitySession, map_db_error, Client};

impl QuerySessions for Client {
    #[tracing::instrument(skip(self))]
    async fn get_user_sessions(
        &self,
        user_id: &Uuid,
    ) -> Result<impl ExactSizeIterator<Item = Session>, CoreError> {
        trace!("getting user sessions");
        let create_id = |id: &Uuid| -> Thing {
            Thing::from((
                Collection::User.to_string().as_str(),
                id.to_string().as_str(),
            ))
        };

        let mut resp = self
            .client
            .query("SELECT * FROM type::table($table) WHERE in = type::record($user)")
            .bind(("table", Collection::UserSession))
            .bind(("user", create_id(user_id)))
            .await
            .map_err(map_db_error)?;
        trace!("mapping entities");

        let res: Vec<DatabaseEntitySession> = resp.take(0).map_err(map_db_error)?;
        event!(Level::INFO, sessions = %res.len(), "found sessions from db");

        let res = res
            .into_iter()
            .map(Session::try_from)
            .collect::<Result<Vec<Session>, CoreError>>()?;

        Ok(res.into_iter())
    }
}
