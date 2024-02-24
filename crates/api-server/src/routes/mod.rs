pub mod middleware;

use axum::response::IntoResponse;

pub async fn handler() -> impl IntoResponse {
    #[cfg(debug_assertions)]
    {
        use crate::SUBSCRIPTION_ENDPOINT;
        use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
        use axum::response::Html;

        Html(playground_source(
            GraphQLPlaygroundConfig::new("/").subscription_endpoint(SUBSCRIPTION_ENDPOINT),
        ))
    }

    #[cfg(not(debug_assertions))]
    {
        format!(
            "{} v{} is live",
            env!("CARGO_CRATE_NAME"),
            env!("CARGO_PKG_VERSION")
        )
    }
}
