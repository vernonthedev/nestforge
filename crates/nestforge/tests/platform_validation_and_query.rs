use axum::{routing::get, Json, Router};
use nestforge::{dto, Query, Validate};
use serde::Deserialize;
use tower::ServiceExt;

#[dto]
struct CreateUserDto {
    #[validate(required, email)]
    email: String,
    #[validate(min_length = 3, max_length = 12)]
    username: String,
    #[validate(min = 18, max = 120)]
    age: u32,
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    page: u32,
}

#[tokio::test]
async fn validate_supports_length_and_numeric_rules() {
    let dto = CreateUserDto {
        email: "bad-email".to_string(),
        username: "ab".to_string(),
        age: 17,
    };

    let errors = dto.validate().expect_err("dto should fail validation");
    let messages = errors
        .errors
        .iter()
        .map(|error| error.message.clone())
        .collect::<Vec<_>>();

    assert!(messages.iter().any(|message| message.contains("valid email")));
    assert!(messages.iter().any(|message| message.contains("at least 3 characters")));
    assert!(messages.iter().any(|message| message.contains("at least 18")));
}

#[tokio::test]
async fn query_extractor_parses_typed_query_parameters() {
    let app = Router::new().route(
        "/search",
        get(|query: Query<SearchQuery>| async move { Json(query.value().page) }),
    );

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/search?page=7")
                .body(axum::body::Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}
