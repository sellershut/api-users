mod account;
mod session;

use std::str::FromStr;

use api_core::{
    api::{CoreError, MutateUsers},
    reexports::uuid::Uuid,
    User, UserType,
};
use surrealdb::sql::{Datetime, Thing};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tracing::{error, event, instrument, trace, Level};

use crate::{
    collections::Collection,
    entity::DatabaseEntityUser,
    map_db_error,
    redis::{cache_keys::CacheKey, PoolLike, PooledConnectionLike},
    Client,
};

impl MutateUsers for Client {
    #[instrument(skip(self), err(Debug))]
    async fn create_user(&self, user: &User) -> Result<User, CoreError> {
        trace!("creating user");
        let input_user = InputUser::from(user);

        let id = Uuid::now_v7().to_string();
        trace!(id = %id, "generated id");

        let item: Option<DatabaseEntityUser> = self
            .client
            .create((Collection::User.to_string(), &id))
            .content(input_user)
            .await
            .map_err(map_db_error)?;
        event!(Level::INFO, id = %id, "user created");

        match item {
            Some(e) => {
                let user = User::try_from(e)?;

                Ok(user)
            }
            None => Err(CoreError::Unreachable),
        }
    }

    #[instrument(skip(self, id), err(Debug))]
    async fn update_user(&self, id: &Uuid, data: &User) -> Result<Option<User>, CoreError> {
        trace!("updating user");
        let id = Thing::from((
            Collection::User.to_string().as_str(),
            id.to_string().as_str(),
        ));

        let mut input_user = InputUser::from(data);
        trace!("calculating updated time");
        let now = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .expect("date time conversion");
        input_user.updated = Some(Datetime::from_str(&now).expect("date time conversion"));

        let item: Option<DatabaseEntityUser> = self
            .client
            .update(id)
            .merge(input_user)
            .await
            .map_err(map_db_error)?;
        event!(Level::INFO, "updated user");

        let res = match item {
            Some(e) => {
                let user = User::try_from(e)?;

                if let Some((ref redis, _ttl)) = self.redis {
                    let user_key = CacheKey::UserById { id: &user.id };
                    let user_key_2 = CacheKey::AllUsers;
                    let user_key_3 = CacheKey::UserByEmail { email: &user.email };
                    trace!(keys = ?[user_key, user_key_2, user_key_3], "resetting cache");

                    let mut redis = redis.get().await.expect("cache from pool");
                    let mut pipeline = redis::Pipeline::new();
                    let refs = pipeline.del(user_key).del(user_key_2);

                    refs.del(user_key_3);

                    if let Err(e) = redis.query_async_pipeline::<()>(pipeline).await {
                        error!("{e}");
                    } else {
                        event!(Level::INFO, keys = ?[user_key, user_key_2, user_key_3], "cache cleared due to update");
                    }
                }
                Some(user)
            }
            None => None,
        };

        Ok(res)
    }

    #[instrument(skip(self, id), err(Debug))]
    async fn delete_user(&self, id: &Uuid) -> Result<Option<User>, CoreError> {
        trace!("deleting user");
        let id = Thing::from((
            Collection::User.to_string().as_str(),
            id.to_string().as_ref(),
        ));

        let res: Option<DatabaseEntityUser> = self.client.delete(id).await.map_err(map_db_error)?;
        event!(Level::INFO, "user deleted");

        let res = match res {
            Some(e) => {
                let user = User::try_from(e)?;
                if let Some((ref redis, _ttl)) = self.redis {
                    let user_key = CacheKey::UserById { id: &user.id };
                    let user_key_2 = CacheKey::AllUsers;
                    let user_key_3 = CacheKey::UserByEmail { email: &user.email };

                    let mut redis = redis.get().await.expect("cache from pool");
                    let mut pipeline = redis::Pipeline::new();
                    let refs = pipeline.del(user_key).del(user_key_2);

                    trace!(keys = ?[user_key, user_key_2, user_key_3], "resetting cache");
                    refs.del(user_key_3);

                    if let Err(e) = redis.query_async_pipeline::<()>(pipeline).await {
                        error!("{e}");
                    }
                }
                Some(user)
            }
            None => None,
        };

        Ok(res)
    }
}

#[derive(serde::Serialize)]
struct InputUser<'a> {
    username: &'a str,
    email: &'a str,
    name: Option<&'a str>,
    avatar: Option<&'a str>,
    #[serde(rename = "type")]
    user_type: UserType,
    updated: Option<Datetime>,
}

impl<'a> From<&'a User> for InputUser<'a> {
    fn from(value: &'a User) -> Self {
        Self {
            username: &value.username,
            email: value.email.as_ref(),
            name: value.name.as_deref(),
            avatar: value.avatar.as_deref(),
            user_type: value.user_type,
            updated: None,
        }
    }
}
