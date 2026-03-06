use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[nestforge::controller("/users")]
#[derive(Default)]
struct EnvelopeController;

#[nestforge::routes]
impl EnvelopeController {
    #[nestforge::get]
    #[allow(dead_code)]
    async fn list() -> nestforge::ApiEnvelopeResult<Vec<String>> {
        Ok(nestforge::ResponseEnvelope::paginated(
            vec!["alice".to_string(), "bob".to_string()],
            1,
            10,
            2,
        ))
    }
}

#[nestforge::module(controllers = [EnvelopeController])]
#[derive(Default)]
struct EnvelopeModule;

impl EnvelopeModule {}

#[tokio::test]
async fn wraps_payloads_in_standard_response_envelopes() {
    let app = nestforge::NestForgeFactory::<EnvelopeModule>::create()
        .expect("factory")
        .into_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/users")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("json");

    assert_eq!(json["success"], true);
    assert_eq!(json["data"][0], "alice");
    assert_eq!(json["meta"]["total"], 2);
}
