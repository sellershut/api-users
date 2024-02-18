use super::create_client;
use anyhow::Result;
use api_core::{
    api::{MutateUsers, QueryUsers},
    reexports::uuid::Uuid,
    User,
};

fn create_user_item() -> User {
    User {
        id: Uuid::now_v7(),
        name: None,
        username: String::from("foobar"),
        email: None,
        avatar: None,
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
    let client = create_client(Some("test-mutation-create"), false, false).await?;

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
    let user = create_user_item();

    let client = create_client(Some("test-mutation-update"), false, false).await?;

    let input = client.create_user(&user).await?;
    let id = input.id;

    let get_by_id = client.get_user_by_id(&input.id).await?;
    assert_eq!(get_by_id, Some(input));

    client.delete_user(&id).await?;

    Ok(())
}

#[tokio::test]
async fn update_user() -> Result<()> {
    let user = create_user_item();

    let client = create_client(Some("test-mutation-update"), false, false).await?;

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
    let user = create_user_item();
    let client = create_client(Some("test-mutation-delete"), false, false).await?;

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
