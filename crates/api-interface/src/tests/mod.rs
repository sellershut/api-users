use std::sync::Arc;

use async_graphql::{
    extensions::{Extension, ExtensionContext, ExtensionFactory, NextPrepareRequest},
    Request, ServerResult,
};

use crate::{ApiSchemaBuilder, DatabaseCredentials};
use async_trait::async_trait;

mod mutation;
mod query;
mod subscription;

struct DummyExtension;

impl ExtensionFactory for DummyExtension {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(TestExtension)
    }
}

struct TestExtension;

#[async_trait]
impl Extension for TestExtension {
    async fn prepare_request(
        &self,
        ctx: &ExtensionContext<'_>,
        request: Request,
        next: NextPrepareRequest<'_>,
    ) -> ServerResult<Request> {
        next.run(ctx, request).await
    }
}

async fn init_schema() -> async_graphql::Schema<
    crate::graphql::query::Query,
    crate::graphql::mutation::Mutation,
    crate::graphql::subscription::Subscription,
> {
    dotenvy::dotenv().ok();
    let db_host = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL");
    let db_host = db_host.replace("http://", "");

    let username = std::env::var("TEST_DATABASE_USERNAME").expect("TEST_DATABASE_USERNAME");
    let password = std::env::var("TEST_DATABASE_PASSWORD").expect("TEST_DATABASE_PASSWORD");
    let db_namespace = std::env::var("TEST_DATABASE_NAMESPACE").expect("TEST_DATABASE_NAMESPACE");
    let db_name = std::env::var("TEST_DATABASE_NAME").expect("TEST_DATABASE_NAME");

    let database_credentials = DatabaseCredentials {
        db_dsn: &db_host,
        db_user: &username,
        db_pass: &password,
        db_ns: &db_namespace,
        db: &db_name,
    };

    ApiSchemaBuilder::new(database_credentials, None, None)
        .await
        .expect("schema created successfully")
        .with_extension(DummyExtension)
        .build()
}
