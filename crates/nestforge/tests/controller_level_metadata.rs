use nestforge::{
    authenticated, controller, roles, routes, AuthIdentity, Container, ModuleDefinition,
    NestForgeFactory,
};
use tower::ServiceExt;

#[controller("/admin")]
struct AdminController;

#[routes]
#[authenticated]
#[roles("admin")]
impl AdminController {
    #[nestforge::get("/dashboard")]
    async fn dashboard() -> nestforge::ApiResult<String> {
        Ok(axum::Json("ok".to_string()))
    }
}

struct AppModule;

impl ModuleDefinition for AppModule {
    fn register(_container: &Container) -> anyhow::Result<()> {
        Ok(())
    }

    fn controllers() -> Vec<axum::Router<Container>> {
        vec![<AdminController as nestforge::ControllerDefinition>::router()]
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
