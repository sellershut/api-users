mod mutation;
mod query;
mod redis;

use crate::Client;
use anyhow::Result;

async fn create_client(
    with_ns: Option<&str>,
    with_redis: bool,
    with_search: bool,
) -> Result<Client> {
    dotenvy::dotenv().ok();

    let db_host = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL");
    let db_host = db_host.replace("http://", "");

    let username = std::env::var("TEST_DATABASE_USERNAME").expect("TEST_DATABASE_USERNAME");
    let password = std::env::var("TEST_DATABASE_PASSWORD").expect("TEST_DATABASE_PASSWORD");
    let db_namespace = std::env::var("TEST_DATABASE_NAMESPACE").expect("TEST_DATABASE_NAMESPACE");
    let db_name = std::env::var("TEST_DATABASE_NAME").expect("TEST_DATABASE_NAME");
    let meilisearch_host = std::env::var("MEILISEARCH_HOST").expect("MEILISEARCH_HOST");
    let meilisearch_api_key = std::env::var("MEILISEARCH_API_KEY").expect("MEILISEARCH_API_KEY");
    let meilisearch_api_key = if meilisearch_api_key.is_empty() {
        None
    } else {
        Some(meilisearch_api_key)
    };

    let redis_host = std::env::var("TEST_REDIS_HOST").expect("TEST_REDIS_HOST");

    let client = Client::try_new(
        &db_host,
        &username,
        &password,
        with_ns.unwrap_or(&db_namespace),
        &db_name,
        if with_redis {
            Some((&redis_host, false, 10, 5000))
        } else {
            None
        },
        if with_search {
            Some((&meilisearch_host, meilisearch_api_key.as_deref()))
        } else {
            None
        },
    )
    .await?;

    Ok(client)
}
