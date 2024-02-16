use crate::{
    redis::{cache_keys::CacheKey, PoolLike, PooledConnectionLike},
    tests::create_client,
    Client,
};
use anyhow::Result;
use api_core::{api::QueryCategories, reexports::uuid::Uuid, User};

async fn check_categories_by_id(client: Client, id: &Uuid, expected_result: bool) -> Result<()> {
    match client.get_category_by_id(id).await {
        Ok(res) => {
            assert_eq!(res.is_some(), expected_result);
        }
        Err(_) => {
            assert!(!expected_result);
        }
    }

    Ok(())
}

async fn check_all(expected_result: bool) -> Result<()> {
    let client = create_client(None, false, false).await?;

    let res = client.get_categories().await;

    assert_eq!(res.is_ok(), expected_result);

    Ok(())
}

async fn check_sub_categories(
    client: Client,
    id: Option<&Uuid>,
    expected_result: bool,
) -> Result<()> {
    match client.get_sub_categories(id).await {
        Ok(categories) => {
            let categories: Vec<_> = categories.collect();
            if !categories.is_empty() {
                for category in categories {
                    assert_eq!(expected_result, category.parent_id.is_none());
                }
            } else {
                assert!(expected_result);
            }
        }
        Err(_) => {
            assert!(!expected_result);
        }
    }

    Ok(())
}

#[tokio::test]
async fn query_by_unavailable_id() -> Result<()> {
    let client = create_client(None, false, false).await?;
    check_categories_by_id(client, &Uuid::now_v7(), false).await?;

    let client = create_client(None, true, false).await?;
    check_categories_by_id(client, &Uuid::now_v7(), false).await?;

    Ok(())
}

#[tokio::test]
async fn query_by_available_id() -> Result<()> {
    let client = create_client(None, false, false).await?;

    let mut res = client
        .client
        .query("SELECT * FROM category LIMIT 5;")
        .await?;

    let resp: Vec<User> = res.take(0)?;

    if let Some(item) = resp.first() {
        check_categories_by_id(client, &item.id, true).await?;
    }

    Ok(())
}

#[tokio::test]
async fn query_with_meilisearch() -> Result<()> {
    let client = create_client(None, true, true).await?;

    if let Some((ref redis, _)) = client.redis {
        let mut redis = redis.get().await?;
        redis.del::<_, ()>(CacheKey::AllUsers).await?;
    }

    let _results: Vec<_> = client.get_categories().await?.collect();
    let _res = client.search("some thing").await.unwrap();
    let _res_parent = client.search_with_parent_name("some thing").await.unwrap();

    Ok(())
}

#[tokio::test]
async fn query_all() -> Result<()> {
    check_all(true).await?;

    Ok(())
}

#[tokio::test]
async fn query_sub_categories() -> Result<()> {
    let client = create_client(None, true, true).await?;
    check_sub_categories(client, None, true).await?;
    Ok(())
}
