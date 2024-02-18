use core::panic;

use async_graphql::Schema;

use crate::graphql::{mutation::Mutation, query::Query, subscription::Subscription};

async fn execute_mutation(
    query: &str,
    schema: &Schema<Query, Mutation, Subscription>,
    mutation: &str,
) -> String {
    let res = schema.execute(query).await;

    dbg!(query);
    dbg!(&res.errors);

    assert!(res.errors.is_empty());

    match res.data {
        async_graphql::Value::Object(value) => {
            dbg!(&value);
            assert_eq!(value.len(), 1);

            let (name, value) = value.first().unwrap();
            assert_eq!(mutation, &name.to_string());

            match value {
                async_graphql::Value::Object(value) => {
                    assert_eq!(value.len(), 1);
                    let (key, value) = value.first().unwrap();
                    assert_eq!(&key.to_string(), "id");
                    match value {
                        async_graphql::Value::String(e) => {
                            assert!(!e.is_empty());
                            format!("\"{e}\"")
                        }
                        _ => {
                            panic!("id returned unexpected type");
                        }
                    }
                }
                _ => {
                    panic!("id returned unexpected type");
                }
            }
        }
        _ => {
            panic!("unexpected value");
        }
    }
}

#[tokio::test]
async fn gql_mutation() {
    let schema = super::init_schema().await;
    use fake::{faker::lorem::en::Word, Fake};
    let name = format!("\"{}\"", Word().fake::<String>());

    let create_mutation = format!(
        r"
            mutation {{
              createUser(input: {{ username: {name}, userType: INDIVIDUAL }}) {{
                id
              }}
            }}
            "
    );

    let id = execute_mutation(&create_mutation, &schema, "createUser").await;
    dbg!("id", &id);

    let update_mutation = format!(
        r"
            mutation {{
              updateUser(id: {id}, input: {{ username: {name}, userType: COMPANY }}) {{
                id
              }}
            }}
            "
    );

    let id_2 = execute_mutation(&update_mutation, &schema, "updateUser").await;
    assert_eq!(&id, &id_2);

    let delete_mutation = format!(
        r"
            mutation {{
              deleteUser(id: {id}) {{
                id
              }}
            }}
            "
    );

    let id_3 = execute_mutation(&delete_mutation, &schema, "deleteUser").await;
    assert_eq!(&id, &id_3);
}
