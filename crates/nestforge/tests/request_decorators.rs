use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

struct CorrelationId;

impl nestforge::RequestDecorator for CorrelationId {
    type Output = String;

    fn extract(
        _ctx: &nestforge::RequestContext,
        parts: &axum::http::request::Parts,
    ) -> Result<Self::Output, nestforge::HttpException> {
        parts
            .headers
            .get("x-correlation-id")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string)
            .ok_or_else(|| nestforge::HttpException::bad_request("Missing x-correlation-id"))
    }
}

#[nestforge::controller("/decorators")]
#[derive(Default)]
struct DecoratorController;

#[nestforge::routes]
impl DecoratorController {
    #[nestforge::get("/correlation")]
    async fn correlation_id(id: nestforge::Decorated<CorrelationId>) -> nestforge::ApiResult<String> {
        Ok(axum::Json(id.into_inner()))
    }
}

#[nestforge::module(controllers = [DecoratorController])]
#[derive(Default)]
struct DecoratorModule;

impl DecoratorModule {}

#[tokio::test]
async fn custom_request_decorators_can_extract_values_from_request_parts() {
    let app = nestforge::NestForgeFactory::<DecoratorModule>::create()
        .expect("factory")
        .into_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/decorators/correlation")
                .header("x-correlation-id", "corr-42")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
}
