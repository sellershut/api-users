use super::create_client;
use anyhow::Result;
use api_core::{
    api::{MutateUsers, QueryUsers},
    reexports::uuid::Uuid,
    User, UserType,
};
use fake::{
    faker::{
        internet::raw::{FreeEmail, Username},
        name::raw::Name,
    },
    locales::EN,
    Fake,
};
use time::OffsetDateTime;

fn create_user_item() -> User {
    User {
        id: Uuid::now_v7(),
        name: Name(EN).fake(),
        username: Username(EN).fake(),
        email: FreeEmail(EN).fake(),
        avatar: None,
        user_type: UserType::Individual,
        phone_number: None,
        created: OffsetDateTime::now_utc(),
        updated: OffsetDateTime::now_utc(),
    }
}

fn check_similarities(source: &User, dest: &User) {
    assert_eq!(source.name, dest.name);
    assert_eq!(source.username, dest.username);
    assert_eq!(source.email, dest.email);
    assert_eq!(source.avatar, dest.avatar);
}

#[tokio::test]
async fn create_user() -> Result<()> {
    dotenvy::dotenv().ok();
    let namespace = std::env::var("TESTS_NS_CREATE")?;

    let client = create_client(Some(&namespace), false, false).await?;

    let all_users = client.get_users().await?;

    let base_count = all_users.count();
    let user = create_user_item();

    let input = client.create_user(&user).await?;

    let updated_users = client.get_users().await?;

    assert_eq!(base_count + 1, updated_users.count());
    check_similarities(&input, &user);

    client.delete_user(&input.id).await?;
    Ok(())
}

#[tokio::test]
async fn create_get_by_id() -> Result<()> {
    dotenvy::dotenv().ok();
    let user = create_user_item();
    let namespace = std::env::var("TESTS_NS_UPDATE")?;

    let client = create_client(Some(&namespace), false, false).await?;

    let input = client.create_user(&user).await?;
    let id = input.id;

    let get_by_id = client.get_user_by_id(&input.id).await?;
    assert_eq!(get_by_id, Some(input));

    client.delete_user(&id).await?;

    Ok(())
}

#[tokio::test]
async fn update_user() -> Result<()> {
    dotenvy::dotenv().ok();
    let user = create_user_item();

    let namespace = std::env::var("TESTS_NS_UPDATE")?;

    let client = create_client(Some(&namespace), false, false).await?;

    let input = client.create_user(&user).await?;

    let mut update = input.clone();
    update.name = Some("FooBar".to_string());

    // This ID does exist
    let update_res = client
        .update_user(&input.id, &update)
        .await?
        .expect("user to exist in db");

    assert_eq!(&update_res.id, &input.id);
    check_similarities(&update, &update_res);

    client.delete_user(&input.id).await?;

    Ok(())
}

#[tokio::test]
async fn delete_user() -> Result<()> {
    dotenvy::dotenv().ok();
    let user = create_user_item();
    let namespace = std::env::var("TESTS_NS_DELETE")?;

    let client = create_client(Some(&namespace), false, false).await?;

    let all_users = client.get_users().await?;

    let base_count = all_users.count();

    let input = client.create_user(&user).await?;
    // delete and check count
    let deleted_user = client
        .delete_user(&input.id)
        .await?
        .expect("user to be deleted");

    assert_eq!(input, deleted_user);

    let final_count = client.get_users().await?.count();
    assert_eq!(base_count, final_count);

    client.delete_user(&input.id).await?;
    Ok(())
}
