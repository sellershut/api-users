use crate::{create_router, state::AppState};
use anyhow::Result;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn http_check() -> Result<()> {
    dotenvy::dotenv().ok();

    let state = AppState::try_from_env()?;
    dbg!(state.database_credentials());

    let router = create_router(state).await?;

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/")
                .method("POST")
                .body(Body::empty())?,
        )
        .await?;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let response = router
        .oneshot(Request::builder().uri("/").body(Body::empty())?)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}
