use nestforge::{
    authenticated, controller, roles, routes, use_exception_filter, AuthIdentity, Container,
    ExceptionFilter, ModuleDefinition, NestForgeFactory, RequestContext,
};
use tower::ServiceExt;

#[controller("/admin")]
struct AdminController;

#[derive(Default)]
struct RewriteForbiddenFilter;

#[routes]
#[authenticated]
#[roles("admin")]
impl AdminController {
    #[nestforge::get("/dashboard")]
    async fn dashboard() -> nestforge::ApiResult<String> {
        Ok(axum::Json("ok".to_string()))
    }
}

impl ExceptionFilter for RewriteForbiddenFilter {
    fn catch(&self, exception: nestforge::HttpException, _ctx: &RequestContext) -> nestforge::HttpException {
        if exception.status == axum::http::StatusCode::FORBIDDEN {
            nestforge::HttpException::forbidden("filtered forbidden")
                .with_optional_request_id(exception.request_id)
        } else {
            exception
        }
    }
}

#[controller("/filtered")]
struct FilteredController;

#[routes]
#[use_exception_filter(RewriteForbiddenFilter)]
impl FilteredController {
    #[nestforge::get("/deny")]
    async fn deny() -> nestforge::ApiResult<String> {
        Err(nestforge::HttpException::forbidden("original forbidden"))
    }
}

struct AppModule;

impl ModuleDefinition for AppModule {
    fn register(_container: &Container) -> anyhow::Result<()> {
        Ok(())
    }

    fn controllers() -> Vec<axum::Router<Container>> {
        vec![
            <AdminController as nestforge::ControllerDefinition>::router(),
            <FilteredController as nestforge::ControllerDefinition>::router(),
        ]
    }
}

#[tokio::test]
async fn controller_level_authenticated_metadata_enforces_runtime_guards() {
    let app = NestForgeFactory::<AppModule>::create()
        .expect("factory should build")
        .into_router();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/admin/dashboard")
                .body(axum::body::Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn controller_level_roles_metadata_enforces_role_checks() {
    let app = NestForgeFactory::<AppModule>::create()
        .expect("factory should build")
        .with_auth_resolver(|token, _container| async move {
            Ok(token.map(|_| AuthIdentity::new("demo-user").with_roles(["viewer"])))
        })
        .into_router();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/admin/dashboard")
                .header(axum::http::header::AUTHORIZATION, "Bearer demo-token")
                .body(axum::body::Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn controller_level_roles_metadata_allows_matching_roles() {
    let app = NestForgeFactory::<AppModule>::create()
        .expect("factory should build")
        .with_auth_resolver(|token, _container| async move {
            Ok(token.map(|_| AuthIdentity::new("demo-user").with_roles(["admin"])))
        })
        .into_router();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/admin/dashboard")
                .header(axum::http::header::AUTHORIZATION, "Bearer demo-token")
                .body(axum::body::Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn controller_level_exception_filters_rewrite_route_failures() {
    let app = NestForgeFactory::<AppModule>::create()
        .expect("factory should build")
        .into_router();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/filtered/deny")
                .body(axum::body::Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), axum::http::StatusCode::FORBIDDEN);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should read");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("response body should be json");
    assert_eq!(payload["message"], "filtered forbidden");
}
