mod account;
mod session;

use api_core::{
    api::{CoreError, MutateUsers},
    reexports::uuid::Uuid,
    User, UserType,
};
use surrealdb::sql::Thing;
use time::OffsetDateTime;
use tracing::instrument;

use crate::{collections::Collections, entity::DatabaseEntityUser, map_db_error, Client};

impl MutateUsers for Client {
    #[instrument(skip(self), err(Debug))]
    async fn create_user(&self, user: &User) -> Result<User, CoreError> {
        let input_user = InputUser::from(user);

        let id = Uuid::now_v7().to_string();
        let item: Option<DatabaseEntityUser> = self
            .client
            .create((Collections::User.to_string(), id))
            .content(input_user)
            .await
            .map_err(map_db_error)?;

        match item {
            Some(e) => User::try_from(e),
            None => Err(CoreError::Unreachable),
        }
    }

    #[instrument(skip(self, id), err(Debug))]
    async fn update_user(&self, id: &Uuid, data: &User) -> Result<Option<User>, CoreError> {
        let id = Thing::from((
            Collections::User.to_string().as_str(),
            id.to_string().as_str(),
        ));

        let input_user = InputUser::from(data);

        let item: Option<DatabaseEntityUser> = self
            .client
            .update(id)
            .content(input_user)
            .await
            .map_err(map_db_error)?;
        let res = match item {
            Some(e) => Some(User::try_from(e)?),
            None => None,
        };

        Ok(res)
    }

    #[instrument(skip(self, id), err(Debug))]
    async fn delete_user(&self, id: &Uuid) -> Result<Option<User>, CoreError> {
        let id = Thing::from((
            Collections::User.to_string().as_str(),
            id.to_string().as_ref(),
        ));

        let res: Option<DatabaseEntityUser> = self.client.delete(id).await.map_err(map_db_error)?;
        let res = match res {
            Some(e) => Some(User::try_from(e)?),
            None => None,
        };

        Ok(res)
    }
}

#[derive(serde::Serialize)]
struct InputUser<'a> {
    username: &'a str,
    email: Option<&'a str>,
    name: Option<&'a str>,
    avatar: Option<&'a str>,
    user_type: UserType,
    created_at: &'a OffsetDateTime,
}

impl<'a> From<&'a User> for InputUser<'a> {
    fn from(value: &'a User) -> Self {
        Self {
            username: &value.username,
            email: value.email.as_deref(),
            name: value.name.as_deref(),
            avatar: value.avatar.as_deref(),
            user_type: value.user_type,
            created_at: &value.created_at,
        }
    }
}
