#[tokio::test]
async fn gql_subscription() -> Result<(), Box<dyn std::error::Error>> {
    let schema = super::init_schema().await;

    let res = schema
        .execute(
            r#"
           subscription {
             categories {
                 id
             }
           }
           "#,
        )
        .await;

    assert!(!res.errors.is_empty());

    Ok(())
}
