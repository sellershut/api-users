#[tokio::test]
async fn gql_query() -> Result<(), Box<dyn std::error::Error>> {
    let schema = super::init_schema().await;

    let res = schema
        .execute(
            r#"
           query {
             users(first: 2) {
               edges{
                 cursor
                 node{
                   id,
                   name
                 }
               },
               pageInfo {
                 hasNextPage,
                 hasPreviousPage
               }
             }
           }
           "#,
        )
        .await;

    assert!(res.errors.is_empty());

    Ok(())
}

#[tokio::test]
async fn gql_query_user() {
    let schema = super::init_schema().await;

    let res = schema
        .execute(
            r#"
           query {
             userByEmail(email: "user@email.com") {
               id,
               name
             }
           } 
           "#,
        )
        .await;

    assert!(res.errors.is_empty());

    let res = schema
        .execute(
            r#"
           query {
             users(first: 2) {
               edges{
                 cursor
                 node{
                   id,
                   name
                 }
               },
               pageInfo {
                 hasNextPage,
                 hasPreviousPage
               }
             }
           }
           "#,
        )
        .await;

    assert!(res.errors.is_empty());
}

#[tokio::test]
async fn gql_query_user_by_id_ok() {
    let schema = super::init_schema().await;

    let res = schema
        .execute(
            r#"
           query {
             userById(id: "018d930d-073c-73c2-b9d6-24f1461c18d3") {
               id,
               name
             }
           }
           "#,
        )
        .await;

    dbg!(&res.errors);
    assert!(res.errors.is_empty());
}
