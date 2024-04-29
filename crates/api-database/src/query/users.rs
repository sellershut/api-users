use api_core::{
    api::{CoreError, QueryUsers},
    reexports::uuid::Uuid,
    Session, User,
};
use meilisearch_sdk::{SearchQuery, SearchResults};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use surrealdb::sql::Thing;
use time::{format_description::well_known::Iso8601, OffsetDateTime};
use tracing::{debug, error, event, instrument, trace, Level};

use crate::{
    collections::Collection,
    entity::{record_id_to_uuid, DatabaseEntityAccountProvider, DatabaseEntityUser},
    map_db_error,
    redis::{cache_keys::CacheKey, redis_query},
    Client,
};

#[tracing::instrument(skip(db))]
async fn db_get_users(
    db: &Client,
    wait_for_completion: bool,
) -> Result<std::vec::IntoIter<User>, CoreError> {
    event!(Level::TRACE, "getting all users");
    let users = if let Some((ref redis, _ttl)) = db.redis {
        let cache_key = CacheKey::AllUsers;
        let users = redis_query::query::<Vec<User>>(cache_key, redis).await;

        if let Some(users) = users {
            event!(Level::INFO, cache_key = %cache_key, user_count = %users.len(), "found users through cache");
            users
        } else {
            trace!("no users found in cache, trying database call");
            debug!(cache_key = %cache_key, "no hits found in cache");
            let users: Vec<DatabaseEntityUser> = db
                .client
                .select(Collection::User)
                .await
                .map_err(map_db_error)?;
            trace!("database query completed. Mapping entities to public types...");
            event!(Level::DEBUG, user_count = %users.len(), "queried database...");

            let users = users
                .into_iter()
                .map(User::try_from)
                .collect::<Result<Vec<User>, CoreError>>()?;
            trace!("entities mapped");

            event!(Level::INFO, user_count = %users.len(), "found users from database");

            if let Err(e) = redis_query::update(cache_key, redis, &users, None).await {
                error!(key = %cache_key, "[redis update]: {e}");
            } else {
                event!(Level::INFO, cache_key= %cache_key, user_count = %users.len(), "cached users from database");
            }
            users
        }
    } else {
        let users: Vec<DatabaseEntityUser> = db
            .client
            .select(Collection::User)
            .await
            .map_err(map_db_error)?;
        event!(Level::DEBUG, user_count = %users.len(), "queried database...");
        trace!("database query completed. Mapping entities to public types...");

        let users = users
            .into_iter()
            .map(User::try_from)
            .collect::<Result<Vec<User>, CoreError>>()?;
        trace!("entities mapped");

        event!(Level::INFO, user_count = %users.len(), "found users from database");
        users
    };

    if let Some(ref client) = db.search_client {
        trace!("indexing users for search");
        let index = client.index("users");
        let pk = "id";
        trace!(primary_key = pk, "adding documents");

        let task = index
            .add_documents(&users, Some(pk))
            .await
            .map_err(|e| CoreError::Other(e.to_string()))?;

        if wait_for_completion {
            if let Err(e) = task.wait_for_completion(client, None, None).await {
                error!("{e}");
            } else {
                event!(Level::INFO, user_count = %users.len(), primary_key = pk, "finished meilisearch indexing for users");
            }
        }
        event!(Level::INFO, user_count = %users.len(), primary_key = pk, "started meilisearch indexing for users");
    }

    Ok(users.into_iter())
}

impl QueryUsers for Client {
    #[instrument(skip(self), err(Debug))]
    async fn get_users(&self) -> Result<impl ExactSizeIterator<Item = User>, CoreError> {
        db_get_users(self, false).await
    }

    #[instrument(skip(self), err(Debug))]
    async fn get_user_by_id(&self, id: &Uuid) -> Result<Option<User>, CoreError> {
        trace!("getting user by id");
        if let Some((ref redis, ttl)) = self.redis {
            let cache_key = CacheKey::UserById { id };
            trace!(cache_key = %cache_key, "checking cache");

            let user = redis_query::query::<Option<User>>(cache_key, redis).await;

            if let Some(user) = user {
                event!(Level::INFO, cache_key = %cache_key, "found user through cache");
                Ok(user)
            } else {
                event!(Level::TRACE, cache_key = %cache_key, "user not found in cache");
                trace!("no users found in cache, trying database call");

                let user: Option<DatabaseEntityUser> = self
                    .client
                    .select((Collection::User.to_string(), id.to_string()))
                    .await
                    .map_err(map_db_error)?;
                event!(Level::DEBUG, user= ?user, "database queried");
                trace!("database query completed. Mapping entity to public type...");

                let user = user.and_then(|f| match User::try_from(f) {
                    Ok(cat) => Some(cat),
                    Err(e) => {
                        error!("{e}");
                        None
                    }
                });
                trace!("entity mapped");

                event!(Level::INFO, user = ?user, "user from database");

                if let Err(e) =
                    redis_query::update(cache_key, redis, user.as_ref(), Some(ttl)).await
                {
                    error!(key = %cache_key, "[redis update]: {e}");
                    event!(Level::ERROR, cache_key = %cache_key, user = ?user, "failed to update cache");
                }
                Ok(user)
            }
        } else {
            let user: Option<DatabaseEntityUser> = self
                .client
                .select((Collection::User.to_string(), id.to_string()))
                .await
                .map_err(map_db_error)?;
            event!(Level::DEBUG, user = ?user, "database queried");
            trace!("database query completed. Mapping entity to public type...");

            let user = user.and_then(|f| match User::try_from(f) {
                Ok(cat) => Some(cat),
                Err(e) => {
                    error!("{e}");
                    None
                }
            });
            trace!("entity mapped");

            event!(Level::INFO, "found user from database");

            Ok(user)
        }
    }

    #[instrument(skip(self), err(Debug))]
    async fn get_user_by_email(
        &self,
        email: impl AsRef<str> + Send + Debug,
    ) -> Result<Option<User>, CoreError> {
        trace!("getting user by id");
        let email = email.as_ref();
        if let Some((ref redis, ttl)) = self.redis {
            let cache_key = CacheKey::UserByEmail { email };
            trace!(cache_key = %cache_key, "checking cache");

            let user = redis_query::query::<Option<User>>(cache_key, redis).await;

            if let Some(Some(user)) = user {
                event!(Level::INFO, cache_key = %cache_key, "found user through cache");

                Ok(Some(user))
            } else {
                event!(Level::TRACE, cache_key = %cache_key, "user not found in cache");
                trace!("no users found in cache, trying database call");

                let mut user = self
                    .client
                    .query("SELECT * FROM type::table($table) WHERE email = type::string($email)")
                    .bind(("table", Collection::User))
                    .bind(("email", email))
                    .await
                    .map_err(map_db_error)?;
                event!(Level::DEBUG, user = ?user, "database queried");
                trace!("database query completed. Mapping entity to public type...");

                let user: Option<DatabaseEntityUser> = user.take(0).map_err(map_db_error)?;
                let user = user.and_then(|f| match User::try_from(f) {
                    Ok(cat) => Some(cat),
                    Err(e) => {
                        error!("{e}");
                        None
                    }
                });
                trace!("entity mapped");
                event!(Level::INFO, user = ?user, "user from database");

                if let Err(e) =
                    redis_query::update(cache_key, redis, user.as_ref(), Some(ttl)).await
                {
                    error!(key = %cache_key, "[redis update]: {e}");
                    event!(Level::ERROR, cache_key = %cache_key, user = ?user, "failed to update cache");
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
            trace!("database query completed. Mapping entity to public type...");

            event!(Level::DEBUG, user = ?user, "database queried");

            let user: Option<DatabaseEntityUser> = user.take(0).map_err(map_db_error)?;
            let user = user.and_then(|f| match User::try_from(f) {
                Ok(cat) => Some(cat),
                Err(e) => {
                    error!("{e}");
                    None
                }
            });
            event!(Level::INFO, user = ?user, "response prepared");

            Ok(user)
        }
    }

    #[instrument(skip(self), err(Debug))]
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
        if let Some((ref redis, ttl)) = self.redis {
            let cache_key = CacheKey::UserByAccount {
                provider,
                provider_account_id,
            };

            let user = redis_query::query::<Option<User>>(cache_key, redis).await;
            if let Some(user) = user {
                event!(Level::INFO, cache_key = %cache_key, "found user through cache");
                Ok(user)
            } else {
                let mut user_response = self
                    .client
                    .query(stmt)
                    .bind(("table", Collection::AccountProvider))
                    .bind(("provider_account_id", provider_account_id))
                    .bind(("provider", provider))
                    .await
                    .map_err(map_db_error)?;
                event!(Level::INFO, cache_key = %cache_key, "database queried");
                let mut user_query: Vec<RetVal> = user_response.take(0).map_err(map_db_error)?;
                trace!("mapping entities");
                let user = if user_query.is_empty() {
                    None
                } else {
                    let a = user_query.swap_remove(0);
                    let user = a.user;
                    Some(User::try_from(user)?)
                };

                if let Err(e) =
                    redis_query::update(cache_key, redis, user.as_ref(), Some(ttl)).await
                {
                    error!(key = %cache_key, "[redis update]: {e}");
                } else {
                    event!(Level::ERROR, cache_key = %cache_key, "cache update failed");
                }
                Ok(user)
            }
        } else {
            let mut user_response = self
                .client
                .query(stmt)
                .bind(("table", Collection::AccountProvider))
                .bind(("provider_account_id", provider_account_id))
                .bind(("provider", provider))
                .await
                .map_err(map_db_error)?;
            event!(Level::INFO, "database queried");

            let mut user_query: Vec<RetVal> = user_response.take(0).map_err(map_db_error)?;
            trace!("mapping entities");
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

    #[instrument(skip(self), err(Debug))]
    async fn search(
        &self,
        query: impl AsRef<str> + Send + Debug,
    ) -> Result<impl ExactSizeIterator<Item = User>, CoreError> {
        if let Some(ref client) = self.search_client {
            let mut index = None;
            for retries in 0..3 {
                trace!("checking user search indexing retry {} of 3", {
                    retries + 1
                });
                if let Ok(idx) = client.get_index("users").await {
                    index = Some(idx);
                    trace!("found search index");
                    break;
                }
                let _users = db_get_users(self, true).await?;
            }
            match index {
                Some(index) => {
                    trace!("searching index");
                    let query = SearchQuery::new(&index).with_query(query.as_ref()).build();

                    let results: SearchResults<User> = index
                        .execute_query(&query)
                        .await
                        .map_err(|e| CoreError::Other(e.to_string()))?;
                    event!(Level::INFO, hits = results.hits.len(), "query results");

                    let search_results: Vec<User> = results
                        .hits
                        .into_iter()
                        .map(|hit| User {
                            id: hit.result.id,
                            name: hit.result.name,
                            username: hit.result.username,
                            email: hit.result.email,
                            avatar: hit.result.avatar,
                            user_type: hit.result.user_type,
                            phone_number: hit.result.phone_number,
                            created: hit.result.created,
                            updated: hit.result.updated,
                        })
                        .collect();

                    Ok(search_results.into_iter())
                }
                None => Err(CoreError::Other(
                    "items could not be indexed for search".into(),
                )),
            }
        } else {
            Err(CoreError::Other(String::from(
                "no client configured for search",
            )))
        }
    }

    #[instrument(skip(self, session_token), err(Debug))]
    async fn get_session_and_user(
        &self,
        session_token: impl AsRef<str> + Send + Debug,
    ) -> Result<Option<(User, Session)>, CoreError> {
        trace!("getting session and user by session token");
        let session_token = session_token.as_ref();

        let mut session = self
            .client
            .query("SELECT in.*,out.*,* FROM type::table($table) WHERE session_token = type::string($session_token)")
            .bind(("table", Collection::UserSession))
            .bind(("session_token", session_token))
            .await
            .map_err(map_db_error)?;
        event!(Level::INFO, "database queried");

        #[derive(Debug, Deserialize)]
        struct Root {
            #[serde(rename = "id")]
            _id: Thing,
            expires_at: String,
            #[serde(rename = "in")]
            in_field: DatabaseEntityUser,
            out: DatabaseEntityAccountProvider,
            session_token: String,
        }

        trace!("mapping entity");
        let user: Option<Root> = session.take(0).map_err(map_db_error)?;

        if let Some(val) = user {
            event!(Level::INFO, id = %val._id, "found user");
            let user_id = record_id_to_uuid(&val.in_field.id)?;
            let session = Session {
                expires_at: OffsetDateTime::parse(&val.expires_at, &Iso8601::DEFAULT).expect(""),
                session_token: val.session_token,
                account_provider: api_core::AccountProvider {
                    id: record_id_to_uuid(&val.out.id)?,
                    name: val.out.name,
                },
                user_id,
            };

            let user = User {
                id: user_id,
                username: val.in_field.username,
                email: val.in_field.email,
                name: val.in_field.name,
                avatar: val.in_field.avatar,
                user_type: val.in_field.user_type,
                phone_number: val.in_field.phone_number,
                created: val.in_field.created,
                updated: val.in_field.updated,
            };

            Ok(Some((user, session)))
        } else {
            event!(Level::INFO, "user not found");
            Ok(None)
        }
    }
}
