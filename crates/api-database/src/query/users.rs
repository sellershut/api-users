use api_core::{
    api::{CoreError, QueryUsers},
    reexports::uuid::Uuid,
    Session, User,
};
use std::{fmt::Debug, str::FromStr};
use surrealdb::sql::Thing;
use time::OffsetDateTime;
use tracing::{debug, error, instrument};

use crate::{
    collections::Collections,
    entity::{record_id_to_uuid, DatabaseEntityUser},
    map_db_error,
    redis::{cache_keys::CacheKey, redis_query},
    Client,
};

async fn db_get_users(db: &Client) -> Result<std::vec::IntoIter<User>, CoreError> {
    let categories = if let Some((ref redis, ttl)) = db.redis {
        let cache_key = CacheKey::AllUsers;
        let users = redis_query::query::<Vec<User>>(cache_key, redis).await;

        if let Some(users) = users {
            users
        } else {
            let users: Vec<DatabaseEntityUser> = db
                .client
                .select(Collections::User)
                .await
                .map_err(map_db_error)?;

            let categories = users
                .into_iter()
                .map(User::try_from)
                .collect::<Result<Vec<User>, CoreError>>()?;

            if let Err(e) = redis_query::update(cache_key, redis, &categories, ttl).await {
                error!(key = %cache_key, "[redis update]: {e}");
            }

            categories
        }
    } else {
        let categories: Vec<DatabaseEntityUser> = db
            .client
            .select(Collections::User)
            .await
            .map_err(map_db_error)?;
        categories
            .into_iter()
            .map(User::try_from)
            .collect::<Result<Vec<User>, CoreError>>()?
    };

    if let Some(ref client) = db.search_client {
        debug!("indexing categories for search");
        let index = client.index("categories");
        index
            .add_documents(&categories, Some("id"))
            .await
            .map_err(|e| CoreError::Other(e.to_string()))?;
    }

    Ok(categories.into_iter())
}

impl QueryUsers for Client {
    #[instrument(skip(self), err(Debug))]
    async fn get_users(&self) -> Result<impl ExactSizeIterator<Item = User>, CoreError> {
        db_get_users(self).await
    }

    #[instrument(skip(self), err(Debug))]
    async fn get_user_by_id(&self, id: &Uuid) -> Result<Option<User>, CoreError> {
        let create_id = |id: &Uuid| -> Thing {
            Thing::from((
                Collections::User.to_string().as_str(),
                id.to_string().as_str(),
            ))
        };

        if let Some((ref redis, ttl)) = self.redis {
            let cache_key = CacheKey::UserById { id };

            let user = redis_query::query::<User>(cache_key, redis).await;

            if user.is_some() {
                Ok(user)
            } else {
                let id = create_id(id);

                let user: Option<DatabaseEntityUser> =
                    self.client.select(id).await.map_err(map_db_error)?;
                let user = user.and_then(|f| match User::try_from(f) {
                    Ok(cat) => Some(cat),
                    Err(e) => {
                        error!("{e}");
                        None
                    }
                });

                if let Err(e) = redis_query::update(cache_key, redis, user.as_ref(), ttl).await {
                    error!(key = %cache_key, "[redis update]: {e}");
                }
                Ok(user)
            }
        } else {
            let id = create_id(id);

            let user: Option<DatabaseEntityUser> =
                self.client.select(id).await.map_err(map_db_error)?;
            let user = user.and_then(|f| match User::try_from(f) {
                Ok(cat) => Some(cat),
                Err(e) => {
                    error!("{e}");
                    None
                }
            });

            Ok(user)
        }
    }

    async fn get_user_by_email(
        &self,
        email: impl AsRef<str> + Send + Debug,
    ) -> Result<Option<User>, CoreError> {
        let email = email.as_ref();
        if let Some((ref redis, ttl)) = self.redis {
            let cache_key = CacheKey::UserByEmail { email };

            let user = redis_query::query::<User>(cache_key, redis).await;

            if user.is_some() {
                Ok(user)
            } else {
                let mut user = self
                    .client
                    .query(format!(
                        "SELECT * FROM {} WHERE email = {email}",
                        Collections::User
                    ))
                    .await
                    .map_err(map_db_error)?;
                let user: Option<DatabaseEntityUser> = user.take(0).map_err(map_db_error)?;
                let user = user.and_then(|f| match User::try_from(f) {
                    Ok(cat) => Some(cat),
                    Err(e) => {
                        error!("{e}");
                        None
                    }
                });

                if let Err(e) = redis_query::update(cache_key, redis, user.as_ref(), ttl).await {
                    error!(key = %cache_key, "[redis update]: {e}");
                }
                Ok(user)
            }
        } else {
            let mut user = self
                .client
                .query(format!(
                    "SELECT * FROM {} WHERE email = {email}",
                    Collections::User
                ))
                .await
                .map_err(map_db_error)?;
            let user: Option<DatabaseEntityUser> = user.take(0).map_err(map_db_error)?;
            let user = user.and_then(|f| match User::try_from(f) {
                Ok(cat) => Some(cat),
                Err(e) => {
                    error!("{e}");
                    None
                }
            });

            Ok(user)
        }
    }

    async fn get_user_by_account(
        &self,
        provider: impl AsRef<str> + Send + Debug,
        provider_account_id: impl AsRef<str> + Send + Debug,
    ) -> Result<Option<User>, CoreError> {
        let provider = provider.as_ref();
        let provider_account_id = provider_account_id.as_ref();
        let stmt = format!("SELECT user_id FROM {} WHERE provider_account_id = {provider_account_id} AND provider = {provider_account_id} FETCH user_id", Collections::Account);
        if let Some((ref redis, ttl)) = self.redis {
            let cache_key = CacheKey::UserByAccount {
                provider,
                provider_account_id,
            };

            let user = redis_query::query::<User>(cache_key, redis).await;

            if user.is_some() {
                Ok(user)
            } else {
                let mut user = self.client.query(stmt).await.map_err(map_db_error)?;
                let user: Option<DatabaseEntityUser> = user.take(0).map_err(map_db_error)?;
                let user = user.and_then(|f| match User::try_from(f) {
                    Ok(cat) => Some(cat),
                    Err(e) => {
                        error!("{e}");
                        None
                    }
                });

                if let Err(e) = redis_query::update(cache_key, redis, user.as_ref(), ttl).await {
                    error!(key = %cache_key, "[redis update]: {e}");
                }
                Ok(user)
            }
        } else {
            let mut user = self.client.query(stmt).await.map_err(map_db_error)?;
            let user: Option<DatabaseEntityUser> = user.take(0).map_err(map_db_error)?;
            let user = user.and_then(|f| match User::try_from(f) {
                Ok(cat) => Some(cat),
                Err(e) => {
                    error!("{e}");
                    None
                }
            });

            Ok(user)
        }
    }

    async fn search(
        &self,
        query: impl AsRef<str> + Send + Debug,
    ) -> Result<impl ExactSizeIterator<Item = User>, CoreError> {
        Ok(vec![].into_iter())
    }

    async fn get_session_and_user(
        &self,
        session_token: impl AsRef<str> + Send + Debug,
    ) -> Result<Option<(User, Session)>, CoreError> {
        let session_token = session_token.as_ref();
        let stmt = format!(
            "SELECT *, user*.*. FROM {} WHERE session_token = {session_token} FETCH user_id",
            Collections::Session
        );
        let mut session = self.client.query(stmt).await.map_err(map_db_error)?;
        let user: Option<serde_json::Value> = session.take(0).map_err(map_db_error)?;
        dbg!(&user);
        let user = user.unwrap();
        match user {
            serde_json::Value::Object(obj) => {
                let mut id = String::default();
                let mut session_token = String::default();
                let mut expires = String::default();
                let mut user: Option<User> = None;

                for (key, value) in obj.iter() {
                    match key.as_str() {
                        "session_token" | "id" | "expires" => {
                            if let serde_json::Value::String(my_id) = value {
                                if key == "id" {
                                    id = my_id.to_owned();
                                } else if key == "session_token" {
                                    session_token = my_id.to_owned();
                                } else if key == "expires" {
                                    expires = my_id.to_owned();
                                }
                            } else {
                                error!("id is {value} which is not a string");
                                unreachable!("this key type should not exist");
                            }
                        }
                        "user" => {
                            if let serde_json::Value::Object(_) = value {
                                user = Some(
                                    serde_json::from_value(value.clone())
                                        .map_err(|e| CoreError::Other(e.to_string()))?,
                                );
                            } else {
                                error!("id is {value} which is not a string");
                                unreachable!("this key type should not exist");
                            }
                        }
                        _ => {
                            error!("{key} should not exist in sessions");
                            unreachable!("this key should not exist");
                        }
                    }
                }
                let id = Thing::from_str(&id).map_err(|_| {
                    CoreError::Database("could not map id to internal type".to_string())
                })?;

                let user = user.expect("user to exist for every session");

                let expires: OffsetDateTime = serde_json::from_str(&expires).unwrap();

                let session = Session {
                    id: record_id_to_uuid(&id)?,
                    user: user.id,
                    expires,
                    session_token,
                };

                Ok(Some((user, session)))
            }
            _ => Err(CoreError::Database(
                "The query returned an unexpected type".to_owned(),
            )),
        }
    }
}
