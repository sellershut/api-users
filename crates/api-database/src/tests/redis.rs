use anyhow::Result;

use crate::redis::{PoolLike, PooledConnectionLike, RedisPool};

async fn get_pool(redis_dsn: &str, max_pool_size: u16, is_cluster: bool) -> RedisPool {
    match is_cluster {
        true => crate::redis::new_redis_pool_clustered(redis_dsn, max_pool_size).await,
        _ => crate::redis::new_redis_pool(redis_dsn, max_pool_size).await,
    }
}

async fn client() -> RedisPool {
    dotenvy::dotenv().ok();

    let redis_dsn = std::env::var("TEST_REDIS_HOST").unwrap_or("redis://localhost:6379".to_owned());

    dbg!(&redis_dsn);

    get_pool(&redis_dsn, 10, false).await
}

#[tokio::test]
async fn redis_query_get_set() {
    let pool = client().await;
    let mut pool = pool.get().await.unwrap();

    for (val, key) in "abcdefghijklmnopqrstuvwxyz".chars().enumerate() {
        let key = key.to_string();
        pool.query_async::<()>(redis::Cmd::set::<String, usize>(key.clone(), val))
            .await
            .unwrap();
        assert_eq!(
            pool.query_async::<usize>(redis::Cmd::get(&key))
                .await
                .unwrap(),
            val
        );
    }
}

#[tokio::test]
async fn redis_get_set() -> Result<()> {
    let pool = client().await;
    let mut pool = pool.get().await.unwrap();
    let key = "IQWOD";
    pool.del::<_, ()>(key).await.unwrap();

    pool.set::<_, &str, ()>(key, "abc").await.unwrap();

    let get = pool.get::<_, String>(key).await.unwrap();
    assert_eq!("abc", get.as_str());

    let res = pool.del::<_, ()>(key).await;

    assert!(res.is_ok());

    Ok(())
}

#[tokio::test]
async fn redis_list_check() -> Result<()> {
    let pool = client().await;
    let mut pool = pool.get().await.unwrap();
    let key = "HJIUN";
    pool.del::<_, ()>(key).await.unwrap();

    let res = pool.lpop::<&str, ()>(key, None).await;
    dbg!(&res);
    assert!(res.is_ok());

    let res = pool.lrange::<&str, ()>(key, 1, 3).await;
    assert!(res.is_ok());

    let res = pool.lrem::<&str, isize, isize>(key, 1, 3).await;
    assert!(res.is_ok());

    let res = pool.pset_ex::<&str, isize, ()>(key, 1, 300).await;
    assert!(res.is_ok());

    let res = pool.rpush::<&str, _, ()>(key, &[1, 2, 4]).await;
    assert!(res.is_err()); //key does not hold a list

    Ok(())
}

#[tokio::test]
async fn redis_sets_check() -> Result<()> {
    let pool = client().await;
    let mut pool = pool.get().await.unwrap();
    let key = "NUIWNIQ";

    let res = pool.zadd::<&str, isize, isize, ()>(key, 1, 2).await;
    assert!(res.is_ok());

    let res = pool
        .zadd_multiple::<&str, usize, usize, ()>(key, &[(1, 1)])
        .await;
    assert!(res.is_ok());

    let res = pool.zpopmin::<&str, Vec<isize>>(key, 1).await;
    assert_eq!(vec![1, 1], res.unwrap());

    let res = pool.zrange_withscores::<&str, Vec<isize>>(key, 1, 2).await;
    assert!(res.is_ok());

    let res = pool
        .zrangebyscore_limit::<&str, isize, isize, ()>(key, 1, 2, 0, 1)
        .await;
    dbg!(&res);
    assert!(res.is_ok());

    Ok(())
}
