use tracing::{debug, error, trace, warn};

use super::{cache_keys::CacheKey, PoolLike, PooledConnectionLike, RedisPool};

#[tracing::instrument]
pub async fn query<T: serde::de::DeserializeOwned>(
    cache_key: CacheKey<'_>,
    redis: &RedisPool,
) -> Option<T> {
    trace!("getting cache worker from pool");
    match redis.get().await {
        Ok(mut redis) => match redis.get::<_, Vec<u8>>(cache_key).await {
            Ok(bytes) => {
                if bytes.is_empty() {
                    warn!("found empty byte array");
                    None
                } else {
                    debug!("deserialising payload");
                    match bincode::deserialize::<T>(&bytes[..]) {
                        Ok(value) => Some(value),
                        Err(decode_err) => {
                            error!("[cache decode]: {decode_err}");
                            None
                        }
                    }
                }
            }
            Err(e) => {
                error!("[redis]: {e}");
                None
            }
        },
        Err(e) => {
            error!("[redis pool]: {e}");
            None
        }
    }
}

#[tracing::instrument]
pub async fn update<T: serde::Serialize + std::fmt::Debug>(
    cache_key: CacheKey<'_>,
    redis: &RedisPool,
    data: T,
    ttl: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!("serialising payload");
    let bytes = bincode::serialize(&data)?;
    debug!("payload serialised");

    trace!("getting cache worker from pool");
    let mut redis = redis.get().await?;
    debug!("got cache worker from pool");

    let res = if let Some(ttl) = ttl {
        debug!("setting cache with expiry");
        redis.pset_ex::<_, _, ()>(cache_key, bytes, ttl).await
    } else {
        debug!("setting cache with no expiry");
        redis.set::<_, _, ()>(cache_key, bytes).await
    };

    if let Err(e) = res {
        error!("[cache update]: {e}");
    }

    Ok(())
}
