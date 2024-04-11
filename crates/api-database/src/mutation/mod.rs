mod account;
mod session;

use api_core::{
    api::{CoreError, MutateUsers},
    reexports::uuid::Uuid,
    User, UserType,
};
use surrealdb::sql::Thing;
use time::OffsetDateTime;
use tracing::{error, instrument};

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
        let input_user = InputUser::from(user);

        let id = Uuid::now_v7().to_string();
        let item: Option<DatabaseEntityUser> = self
            .client
            .create((Collection::User.to_string(), id))
            .content(input_user)
            .await
            .map_err(map_db_error)?;

        match item {
            Some(e) => {
                let user = User::try_from(e)?;

                if let Some((ref redis, _ttl)) = self.redis {
                    let user_key = CacheKey::AllUsers;

                    let mut redis = redis.get().await.unwrap();

                    if let Err(e) = redis.del::<_, ()>(user_key).await {
                        error!("{e}");
                    }
                }
                Ok(user)
            }
            None => Err(CoreError::Unreachable),
        }
    }

    #[instrument(skip(self, id), err(Debug))]
    async fn update_user(&self, id: &Uuid, data: &User) -> Result<Option<User>, CoreError> {
        let id = Thing::from((
            Collection::User.to_string().as_str(),
            id.to_string().as_str(),
        ));

        let mut input_user = InputUser::from(data);
        input_user.updated = Some(OffsetDateTime::now_utc().unix_timestamp_nanos());

        let item: Option<DatabaseEntityUser> = self
            .client
            .update(id)
            .content(input_user)
            .await
            .map_err(map_db_error)?;
        let res = match item {
            Some(e) => {
                let user = User::try_from(e)?;

                if let Some((ref redis, _ttl)) = self.redis {
                    let user_key = CacheKey::UserById { id: &user.id };
                    let user_key_2 = CacheKey::AllUsers;

                    let mut redis = redis.get().await.unwrap();
                    let mut pipeline = redis::Pipeline::new();
                    let refs = pipeline.del(user_key).del(user_key_2);

                    if let Some(ref email) = user.email {
                        let user_key_3 = CacheKey::UserByEmail { email };
                        refs.del(user_key_3);
                    }

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

    #[instrument(skip(self, id), err(Debug))]
    async fn delete_user(&self, id: &Uuid) -> Result<Option<User>, CoreError> {
        let id = Thing::from((
            Collection::User.to_string().as_str(),
            id.to_string().as_ref(),
        ));

        let res: Option<DatabaseEntityUser> = self.client.delete(id).await.map_err(map_db_error)?;
        let res = match res {
            Some(e) => {
                let user = User::try_from(e)?;
                if let Some((ref redis, _ttl)) = self.redis {
                    let user_key = CacheKey::UserById { id: &user.id };
                    let user_key_2 = CacheKey::AllUsers;

                    let mut redis = redis.get().await.unwrap();
                    let mut pipeline = redis::Pipeline::new();
                    let refs = pipeline.del(user_key).del(user_key_2);

                    if let Some(ref email) = user.email {
                        let user_key_3 = CacheKey::UserByEmail { email };
                        refs.del(user_key_3);
                    }

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
    email: Option<&'a str>,
    name: Option<&'a str>,
    avatar: Option<&'a str>,
    #[serde(rename = "type")]
    user_type: UserType,
    updated: Option<i128>,
}

impl<'a> From<&'a User> for InputUser<'a> {
    fn from(value: &'a User) -> Self {
        Self {
            username: &value.username,
            email: value.email.as_deref(),
            name: value.name.as_deref(),
            avatar: value.avatar.as_deref(),
            user_type: value.user_type,
            updated: None,
        }
    }
}
