mod async_graphql;
mod db;

use crate::{tests::db::SampleDbSend, User};

use self::db::SampleDb;
use uuid::Uuid;

fn create_user() -> User {
    User {
        id: Uuid::now_v7(),
        name: Some(String::from("Something")),
        username: String::from("foobar"),
        email: None,
        avatar: None,
    }
}

#[test]
fn encode() {
    let category = create_user();

    let json = serde_json::to_string(&category).unwrap();
    dbg!(&json);
    let bytes = bincode::serialize(&category).unwrap();

    let value = serde_json::from_str::<User>(&json);
    dbg!(&value);

    assert!(value.is_ok());
    let val: User = bincode::deserialize(&bytes[..]).unwrap();
    assert_eq!(val, category);
}

#[test]
fn deserialise_list() {
    let category = create_user();

    let category_2 = User {
        id: Uuid::now_v7(),
        name: Some("Something".into()),
        username: "barfoo".into(),
        email: Some("user@email.com".into()),
        avatar: None,
    };

    let categories = vec![category, category_2];

    let str_val = serde_json::to_string(&categories);

    let bytes = bincode::serialize(&categories).unwrap();

    let source = bincode::deserialize::<Vec<User>>(&bytes[..]).unwrap();

    dbg!(&str_val);

    assert!(str_val.is_ok());
    assert_eq!(source, categories);
}

#[tokio::test]
async fn trait_blank_queries() {
    use crate::api::LocalQueryUsers;

    let db = SampleDb.get_users().await;
    assert!(db.is_ok());

    let generated_id = Uuid::now_v7();
    let db = SampleDb.get_user_by_id(&generated_id).await;
    assert!(db.is_ok());

    let db = SampleDb.get_user_by_email("user@email.com").await;
    assert!(db.is_ok());

    let db = SampleDb.get_session_and_user("").await;
    assert!(db.is_ok());
}

#[tokio::test]
async fn trait_blank_mutations() {
    use crate::api::LocalMutateUsers;

    let user = create_user();

    let db = SampleDb.create_user(&user).await;
    assert!(db.is_ok());

    let id = Uuid::now_v7();
    let db = SampleDb.update_user(&id, &user).await;
    assert!(db.is_ok());

    let db = SampleDb.delete_user(&id).await;
    assert!(db.is_ok());
}

#[tokio::test]
async fn mutation_returns_send() {
    use crate::api::MutateUsers;

    let category = create_user();

    let id = Uuid::now_v7();
    let db = SampleDbSend.create_user(&category).await;
    assert!(db.is_ok());

    let db = SampleDbSend.update_user(&id, &category).await;
    assert!(db.is_ok());

    let db = SampleDbSend.delete_user(&id).await;
    assert!(db.is_ok());
}

#[tokio::test]
async fn query_returns_send() {
    use crate::api::QueryUsers;

    let db = SampleDbSend.get_users().await;
    assert!(db.is_ok());

    let generated_id = Uuid::now_v7();
    let db = SampleDbSend.get_user_by_id(&generated_id).await;
    assert!(db.is_ok());

    let db = SampleDbSend.get_user_by_email("").await;
    assert!(db.is_ok());

    let db = SampleDbSend.get_user_by_account("", "").await;
    assert!(db.is_ok());
}
