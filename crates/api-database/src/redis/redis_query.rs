use tracing::error;

use super::{cache_keys::CacheKey, PoolLike, PooledConnectionLike, RedisPool};

pub async fn query<T: serde::de::DeserializeOwned>(
    cache_key: CacheKey<'_>,
    redis: &RedisPool,
) -> Option<T> {
    match redis.get().await {
        Ok(mut redis) => match redis.get::<_, Vec<u8>>(cache_key).await {
            Ok(bytes) => {
                if bytes.is_empty() {
                    None
                } else {
                    match bincode::deserialize::<T>(&bytes[..]) {
                        Ok(value) => Some(value),
                        Err(decode_err) => {
                            error!(key = %cache_key, "[cache decode]: {decode_err}");
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

pub async fn update<T: serde::Serialize>(
    cache_key: CacheKey<'_>,
    redis: &RedisPool,
    data: T,
    ttl: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = bincode::serialize(&data)?;

    let mut redis = redis.get().await?;

    if let Err(e) = redis.pset_ex::<_, _, ()>(cache_key, bytes, ttl).await {
        error!(key = %cache_key,"[cache update]: {e}");
    }

    Ok(())
}
