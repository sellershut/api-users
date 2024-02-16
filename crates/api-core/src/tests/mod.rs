mod async_graphql;
mod db;

use crate::{tests::db::SampleDbSend, User};

use self::db::SampleDb;
use uuid::Uuid;

fn create_category() -> User {
    User {
        id: Uuid::now_v7(),
        name: String::from("Something"),
        sub_categories: vec![],
        image_url: None,
        parent_id: None,
    }
}

#[test]
fn encode() {
    let category = create_category();

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
    let category = create_category();

    let category_2 = User {
        id: Uuid::now_v7(),
        name: "Something".into(),
        sub_categories: vec![],
        image_url: None,
        parent_id: None,
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

    let db = SampleDb.get_categories().await;
    assert!(db.is_ok());

    let generated_id = Uuid::now_v7();
    let mut id = None;
    let db = SampleDb.get_sub_categories(id).await;
    assert!(db.is_ok());

    id = Some(&generated_id);
    let db = SampleDb.get_sub_categories(id).await;
    assert!(db.is_ok());

    let db = SampleDb.get_category_by_id(&generated_id).await;
    assert!(db.is_ok());
}

#[tokio::test]
async fn trait_blank_mutations() {
    use crate::api::LocalMutateUsers;

    let category = create_category();

    let db = SampleDb.create_category(&category).await;
    assert!(db.is_ok());

    let id = Uuid::now_v7();
    let db = SampleDb.update_category(&id, &category).await;
    assert!(db.is_ok());

    let db = SampleDb.delete_category(&id).await;
    assert!(db.is_ok());
}

#[tokio::test]
async fn mutation_returns_send() {
    use crate::api::MutateCategories;

    let category = create_category();

    let id = Uuid::now_v7();
    let db = SampleDbSend.create_category(&category).await;
    assert!(db.is_ok());

    let db = SampleDbSend.update_category(&id, &category).await;
    assert!(db.is_ok());

    let db = SampleDbSend.delete_category(&id).await;
    assert!(db.is_ok());
}

#[tokio::test]
async fn query_returns_send() {
    use crate::api::QueryCategories;

    let db = SampleDbSend.get_categories().await;
    assert!(db.is_ok());

    let generated_id = Uuid::now_v7();
    let mut id = None;
    let db = SampleDbSend.get_sub_categories(id).await;
    assert!(db.is_ok());

    id = Some(&generated_id);
    let db = SampleDbSend.get_sub_categories(id).await;
    assert!(db.is_ok());

    let db = SampleDbSend.get_category_by_id(&generated_id).await;
    assert!(db.is_ok());
}
