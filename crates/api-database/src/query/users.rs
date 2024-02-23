use api_core::{
    api::{CoreError, QueryUsers},
    reexports::uuid::Uuid,
    Session, User,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use surrealdb::{opt::RecordId, sql::Thing};
use time::OffsetDateTime;
use tracing::{debug, error, instrument, trace};

use crate::{
    collections::Collection,
    entity::{record_id_to_uuid, DatabaseEntityUser},
    map_db_error,
    redis::{cache_keys::CacheKey, redis_query},
    Client,
};

async fn db_get_users(db: &Client) -> Result<std::vec::IntoIter<User>, CoreError> {
    let users = if let Some((ref redis, _ttl)) = db.redis {
        let cache_key = CacheKey::AllUsers;
        let users = redis_query::query::<Vec<User>>(cache_key, redis).await;

        if let Some(users) = users {
            users
        } else {
            let users: Vec<DatabaseEntityUser> = db
                .client
                .select(Collection::User)
                .await
                .map_err(map_db_error)?;

            let users = users
                .into_iter()
                .map(User::try_from)
                .collect::<Result<Vec<User>, CoreError>>()?;

            if let Err(e) = redis_query::update(cache_key, redis, &users, None).await {
                error!(key = %cache_key, "[redis update]: {e}");
            }

            users
        }
    } else {
        let users: Vec<DatabaseEntityUser> = db
            .client
            .select(Collection::User)
            .await
            .map_err(map_db_error)?;
        users
            .into_iter()
            .map(User::try_from)
            .collect::<Result<Vec<User>, CoreError>>()?
    };

    if let Some(ref client) = db.search_client {
        debug!("indexing users for search");
        let index = client.index("users");
        index
            .add_documents(&users, Some("id"))
            .await
            .map_err(|e| CoreError::Other(e.to_string()))?;
    }

    Ok(users.into_iter())
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
                Collection::User.to_string().as_str(),
                id.to_string().as_str(),
            ))
        };

        if let Some((ref redis, _ttl)) = self.redis {
            let cache_key = CacheKey::UserById { id };

            let user = redis_query::query::<Option<User>>(cache_key, redis).await;

            if let Some(user) = user {
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

                if let Err(e) = redis_query::update(cache_key, redis, user.as_ref(), None).await {
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
        if let Some((ref redis, _ttl)) = self.redis {
            let cache_key = CacheKey::UserByEmail { email };

            let user = redis_query::query::<Option<User>>(cache_key, redis).await;

            if let Some(user) = user {
                Ok(user)
            } else {
                let mut user = self
                    .client
                    .query("SELECT * FROM type::table($table) WHERE email = type::string($email)")
                    .bind(("table", Collection::User))
                    .bind(("email", email))
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

                if let Err(e) = redis_query::update(cache_key, redis, user.as_ref(), None).await {
                    error!(key = %cache_key, "[redis update]: {e}");
                }
                Ok(user)
            }
        } else {
            let mut user = self
                .client
                .query("SELECT * FROM type::table($table) WHERE email = type::string($email)")
                .bind(("table", Collection::User))
                .bind(("email", email))
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

        #[derive(Deserialize, Serialize)]
        struct RetVal {
            user: DatabaseEntityUser,
        }

        let stmt = "SELECT user FROM {} WHERE provider_account_id = type::string($provider_account_id) AND provider = type::string($provider) FETCH user";
        if let Some((ref redis, _ttl)) = self.redis {
            let cache_key = CacheKey::UserByAccount {
                provider,
                provider_account_id,
            };

            let user = redis_query::query::<Option<User>>(cache_key, redis).await;
            if let Some(user) = user {
                Ok(user)
            } else {
                let mut user_response = self
                    .client
                    .query(stmt)
                    .bind(("table", Collection::Account))
                    .bind(("provider_account_id", provider_account_id))
                    .bind(("provider", provider))
                    .await
                    .map_err(map_db_error)?;
                let mut user_query: Vec<RetVal> = user_response.take(0).map_err(map_db_error)?;
                let user = if user_query.is_empty() {
                    None
                } else {
                    let a = user_query.swap_remove(0);
                    let user = a.user;
                    Some(User::try_from(user)?)
                };

                if let Err(e) = redis_query::update(cache_key, redis, user.as_ref(), None).await {
                    error!(key = %cache_key, "[redis update]: {e}");
                }
                Ok(user)
            }
        } else {
            let mut user_response = self
                .client
                .query(stmt)
                .bind(("table", Collection::Account))
                .bind(("provider_account_id", provider_account_id))
                .bind(("provider", provider))
                .await
                .map_err(map_db_error)?;
            let mut user_query: Vec<RetVal> = user_response.take(0).map_err(map_db_error)?;
            let user = if user_query.is_empty() {
                None
            } else {
                let a = user_query.swap_remove(0);
                let user = a.user;
                Some(User::try_from(user)?)
            };

            Ok(user)
        }
    }

    async fn search(
        &self,
        _query: impl AsRef<str> + Send + Debug,
    ) -> Result<impl ExactSizeIterator<Item = User>, CoreError> {
        Ok(vec![].into_iter())
    }

    async fn get_session_and_user(
        &self,
        session_token: impl AsRef<str> + Send + Debug,
    ) -> Result<Option<(User, Session)>, CoreError> {
        let session_token = session_token.as_ref();
        let stmt = "SELECT *, user FROM type::table($table) WHERE session_token = type::string($token) FETCH user";
        let mut session = self
            .client
            .query(stmt)
            .bind(("table", Collection::Session))
            .bind(("token", session_token))
            .await
            .map_err(map_db_error)?;
        let user: Option<serde_json::Value> = session.take(0).map_err(map_db_error)?;
        trace!("user: {user:?}");

        if let Some(user) = user {
            match user {
                serde_json::Value::Object(obj) => {
                    let mut id = None;
                    let mut expires: Option<OffsetDateTime> = None;
                    let mut session_token = String::default();
                    let mut user: Option<DatabaseEntityUser> = None;

                    for (key, value) in obj.into_iter() {
                        match key.as_str() {
                            "id" => {
                                trace!("deserialising id: {value:?}");
                                let id_val: RecordId =
                                    serde_json::from_value(value).map_err(|_| {
                                        CoreError::Other("deserialise session id".to_owned())
                                    })?;
                                trace!("id ok");
                                id = Some(id_val);
                            }
                            "session_token" => {
                                trace!("deserialising session: {value:?}");
                                if let serde_json::Value::String(my_id) = value {
                                    session_token = my_id;
                                } else {
                                    error!(key = %key.as_str(), "{value} which is not a string");
                                }
                                trace!("session ok");
                            }
                            "expires_at" => {
                                trace!("sorting time: {value:?}");
                                expires = Some(serde_json::from_value(value).map_err(|_| {
                                    CoreError::Other("Could not deserialise time".to_owned())
                                })?);
                                trace!("time ok");
                            }
                            "user" => {
                                trace!("deserialising user: {value:?}");
                                if let serde_json::Value::Object(_) = value {
                                    user = Some(
                                        serde_json::from_value(value)
                                            .map_err(|e| CoreError::Other(e.to_string()))?,
                                    );
                                } else {
                                    error!("{key} should not exist in sessions");
                                    unreachable!("this key type should not exist");
                                }
                                trace!("user ok");
                            }
                            _ => {
                                error!("{key} should not exist in sessions");
                                unreachable!("this key should not exist");
                            }
                        }
                    }
                    let id = id.expect("to exist");

                    let user = user.expect("user to exist for every session");
                    let id = record_id_to_uuid(&id)?;

                    let user = User::try_from(user)?;
                    trace!("after {}", id);

                    let session = Session {
                        id,
                        user: user.id,
                        expires_at: expires.expect("to exist"),
                        session_token,
                    };

                    Ok(Some((user, session)))
                }
                _ => Err(CoreError::Database(
                    "The query returned an unexpected type".to_owned(),
                )),
            }
        } else {
            Ok(None)
        }
    }
}
