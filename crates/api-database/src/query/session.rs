use api_core::{
    api::{CoreError, QuerySessions},
    Session,
};
use surrealdb::sql::Thing;
use uuid::Uuid;

use crate::{collections::Collections, entity::DatabaseEntitySession, map_db_error, Client};

impl QuerySessions for Client {
    async fn get_user_sessions(
        &self,
        user_id: &Uuid,
    ) -> Result<impl ExactSizeIterator<Item = Session>, CoreError> {
        let create_id = |id: &Uuid| -> Thing {
            Thing::from((
                Collections::User.to_string().as_str(),
                id.to_string().as_str(),
            ))
        };

        let mut resp = self
            .client
            .query(format!(
                "SELECT * FROM {} WHERE user = {}",
                Collections::Session,
                create_id(user_id)
            ))
            .await
            .map_err(map_db_error)?;

        let res: Vec<DatabaseEntitySession> = resp.take(0).map_err(map_db_error)?;

        let res = res
            .into_iter()
            .map(Session::try_from)
            .collect::<Result<Vec<Session>, CoreError>>()?;

        Ok(res.into_iter())
    }
}
