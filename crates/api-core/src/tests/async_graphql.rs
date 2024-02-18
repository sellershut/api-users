use async_graphql::{EmptySubscription, Object, Schema};

use crate::User;

use super::create_user;

struct Root;

#[Object]
impl Root {
    async fn output(&self) -> User {
        create_user()
    }

    async fn input(&self, category: User) -> User {
        category
    }
}

#[tokio::test]
async fn gql_query() {
    let schema = Schema::new(Root, Root, EmptySubscription);

    let res = schema
        .execute(
            r#"
              query {
                output {
                  name
                }
              }
            "#,
        )
        .await;

    dbg!(&res);

    assert!(res.errors.is_empty());
}

#[tokio::test]
async fn gql_mutation() {
    let schema = Schema::new(Root, Root, EmptySubscription);

    let res = schema
        .execute(
            r#"
              mutation {
                input (user: {username: "Lorem"}) {
                  username
                }
              }
            "#,
        )
        .await;

    dbg!(&res);

    assert!(res.errors.is_empty());
}
