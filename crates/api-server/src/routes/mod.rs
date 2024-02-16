pub mod middleware;

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use axum::response::{Html, IntoResponse};

use crate::SUBSCRIPTION_ENDPOINT;

pub async fn handler() -> impl IntoResponse {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/").subscription_endpoint(SUBSCRIPTION_ENDPOINT),
    ))
}
