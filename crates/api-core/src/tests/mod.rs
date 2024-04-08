mod async_graphql;
mod db;

use crate::{tests::db::SampleDbSend, User, UserType};

use self::db::SampleDb;
use time::OffsetDateTime;
use uuid::Uuid;

fn create_user() -> User {
    User {
        id: Uuid::now_v7(),
        username: String::from("foobar"),
        email: None,
        name: Some(String::from("Something")),
        avatar: None,
        user_type: UserType::Individual,
        phone_number: None,
        created: OffsetDateTime::now_utc(),
        updated: None,
    }
}

#[test]
fn encode() {
    let user = create_user();

    let json = serde_json::to_string(&user).unwrap();
    dbg!(&json);
    let bytes = bincode::serialize(&user).unwrap();

    let value = serde_json::from_str::<User>(&json);
    dbg!(&value);

    assert!(value.is_ok());
    let val: User = bincode::deserialize(&bytes[..]).unwrap();
    assert_eq!(val, user);
}

#[test]
fn deserialise_list() {
    let user = create_user();

    let user_2 = User {
        id: Uuid::now_v7(),
        name: Some("Something".into()),
        username: "barfoo".into(),
        email: Some("user@email.com".into()),
        avatar: None,
        user_type: UserType::Individual,
        phone_number: None,
        created: OffsetDateTime::now_utc(),
        updated: None,
    };

    let users = vec![user, user_2];

    let str_val = serde_json::to_string(&users);

    let bytes = bincode::serialize(&users).unwrap();

    let source = bincode::deserialize::<Vec<User>>(&bytes[..]).unwrap();

    dbg!(&str_val);

    assert!(str_val.is_ok());
    assert_eq!(source, users);
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

    let user = create_user();

    let id = Uuid::now_v7();
    let db = SampleDbSend.create_user(&user).await;
    assert!(db.is_ok());

    let db = SampleDbSend.update_user(&id, &user).await;
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
