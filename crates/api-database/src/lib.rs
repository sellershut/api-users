use api_core::api::CoreError;
use thiserror::Error;

mod collections;
pub(crate) mod entity;
mod mutation;
mod query;
mod redis;

use surrealdb::{
    engine::remote::ws::{Client as SurrealClient, Ws},
    opt::auth::Root,
    Surreal,
};
use tracing::{instrument, trace};

use self::redis::RedisPool;

pub(crate) fn map_db_error(error: surrealdb::Error) -> CoreError {
    CoreError::Database(error.to_string())
}

pub struct Client {
    client: Surreal<SurrealClient>,
    redis: Option<(RedisPool, u64)>,
    search_client: Option<meilisearch_sdk::client::Client>,
}

impl Client {
    #[instrument(skip_all)]
    pub async fn try_new(
        dsn: &str,
        username: &str,
        password: &str,
        namespace: &str,
        database: &str,
        redis: Option<(&str, bool, u16, u64)>,
        meilisearch: Option<(&str, Option<&str>)>,
    ) -> Result<Self, ClientError> {
        trace!("connecting to database");
        let db = Surreal::new::<Ws>(dsn).await?;

        // Signin as a namespace, database, or root user
        db.signin(Root { username, password }).await?;

        db.use_ns(namespace).use_db(database).await?;

        Ok(Client {
            client: db,
            search_client: match meilisearch {
                Some((host, api_key)) => Some(
                    meilisearch_sdk::client::Client::new(host, api_key)
                        .map_err(|e| ClientError::Other(e.to_string()))?,
                ),
                None => None,
            },

            redis: match redis {
                Some((dsn, clustered, size, ttl)) => Some((
                    if clustered {
                        redis::new_redis_pool_clustered(dsn, size).await
                    } else {
                        redis::new_redis_pool(dsn, size).await
                    },
                    ttl,
                )),
                None => None,
            },
        })
    }
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("database engine error")]
    Engine(#[from] surrealdb::Error),
    #[error("the data for key `{0}` is not available")]
    Other(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("unknown data store error")]
    Unknown,
}

#[cfg(test)]
mod tests;
