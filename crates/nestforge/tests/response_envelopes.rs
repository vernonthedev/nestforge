use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use nestforge::{ApiEnvelopeResult, ResponseEnvelope};
use tower::ServiceExt;

#[derive(Default)]
struct EnvelopeController;

#[nestforge::controller("/users")]
impl EnvelopeController {
    #[nestforge::get]
    async fn list() -> ApiEnvelopeResult<Vec<String>> {
        Ok(ResponseEnvelope::paginated(
            vec!["alice".to_string(), "bob".to_string()],
            1,
            10,
            2,
        ))
    }
}

#[derive(Default)]
struct EnvelopeModule;

#[nestforge::module(controllers = [EnvelopeController])]
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
