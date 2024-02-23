use crate::{collections::Collection, entity::DatabaseEntityUser, tests::create_client, Client};
use anyhow::Result;
use api_core::{api::QueryUsers, reexports::uuid::Uuid, User};

async fn check_users_by_id(client: Client, id: &Uuid, expected_result: bool) -> Result<()> {
    match client.get_user_by_id(id).await {
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

    let res = client.get_users().await;

    assert_eq!(res.is_ok(), expected_result);

    Ok(())
}

#[tokio::test]
async fn query_by_unavailable_id() -> Result<()> {
    let client = create_client(None, false, false).await?;
    check_users_by_id(client, &Uuid::now_v7(), false).await?;

    let client = create_client(None, true, false).await?;
    check_users_by_id(client, &Uuid::now_v7(), false).await?;

    Ok(())
}

#[tokio::test]
async fn query_by_available_id() -> Result<()> {
    let client = create_client(None, false, false).await?;

    let mut res = client
        .client
        .query(format!("SELECT * FROM {} LIMIT 5;", Collection::User))
        .await?;

    let resp: Vec<DatabaseEntityUser> = res.take(0)?;
    let resp: Result<Vec<User>, _> = resp.into_iter().map(|db| User::try_from(db)).collect();

    let values = resp?;

    if let Some(item) = values.first() {
        check_users_by_id(client, &item.id, true).await?;
    }

    Ok(())
}

#[tokio::test]
async fn query_all() -> Result<()> {
    check_all(true).await?;

    Ok(())
}
