use nestforge::{
    controller, routes, Container, ControllerDefinition, HttpException, ModuleDefinition, Pipe,
    PipedBody, PipedParam, PipedQuery, RequestContext,
};
use serde::Deserialize;
use tower::ServiceExt;

struct TrimmedSlugPipe;

impl Pipe<String> for TrimmedSlugPipe {
    type Output = String;

    fn transform(value: String, _ctx: &RequestContext) -> Result<Self::Output, HttpException> {
        let slug = value.trim().to_lowercase();
        if slug.is_empty() {
            return Err(HttpException::bad_request("slug is required"));
        }
        Ok(slug)
    }
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

struct SearchPipe;

impl Pipe<SearchQuery> for SearchPipe {
    type Output = String;

    fn transform(value: SearchQuery, _ctx: &RequestContext) -> Result<Self::Output, HttpException> {
        let query = value.q.trim().to_string();
        if query.len() < 2 {
            return Err(HttpException::bad_request("query must be at least 2 chars"));
        }
        Ok(query)
    }
}

#[derive(Deserialize)]
struct CreatePayload {
    name: String,
}

struct CreatePipe;

impl Pipe<CreatePayload> for CreatePipe {
    type Output = String;

    fn transform(
        value: CreatePayload,
        _ctx: &RequestContext,
    ) -> Result<Self::Output, HttpException> {
        let name = value.name.trim().to_string();
        if name.is_empty() {
            return Err(HttpException::bad_request("name is required"));
        }
        Ok(name)
    }
}

#[controller("/pipes")]
struct PipesController;

#[routes]
impl PipesController {
    #[nestforge::get("/slug/{value}")]
    async fn slug(value: PipedParam<String, TrimmedSlugPipe>) -> Result<String, HttpException> {
        Ok(value.into_inner())
    }

    #[nestforge::get("/search")]
    async fn search(query: PipedQuery<SearchQuery, SearchPipe>) -> Result<String, HttpException> {
        Ok(query.into_inner())
    }

    #[nestforge::post("/body")]
    async fn body(name: PipedBody<CreatePayload, CreatePipe>) -> Result<String, HttpException> {
        Ok(name.into_inner())
    }
}

struct PipesModule;

impl ModuleDefinition for PipesModule {
    fn register(_container: &Container) -> anyhow::Result<()> {
        Ok(())
    }

    fn controllers() -> Vec<axum::Router<Container>> {
        vec![PipesController::router()]
    }
}

#[tokio::test]
async fn piped_param_transforms_route_input() {
    let app = nestforge::NestForgeFactory::<PipesModule>::create()
        .expect("factory should build")
        .into_router();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/pipes/slug/  Hello-World  ")
                .body(axum::body::Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn piped_query_can_reject_invalid_query_values() {
    let app = nestforge::NestForgeFactory::<PipesModule>::create()
        .expect("factory should build")
        .into_router();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/pipes/search?q=a")
                .body(axum::body::Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn piped_body_transforms_json_payloads() {
    let app = nestforge::NestForgeFactory::<PipesModule>::create()
        .expect("factory should build")
        .into_router();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/pipes/body")
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(axum::body::Body::from(
                    serde_json::json!({ "name": "  Vernon  " }).to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}
