#![cfg(feature = "openapi")]

use nestforge::{
    openapi_doc_for_module, openapi_docs_router_for_module_with_config, Container,
    ModuleDefinition, OpenApiConfig, OpenApiUi,
};
use tower::util::ServiceExt;

#[nestforge::controller("/users")]
struct UsersController;

#[nestforge::routes]
#[nestforge::tag("controller-users")]
#[nestforge::authenticated]
#[nestforge::roles("admin")]
impl UsersController {
    #[nestforge::get("/")]
    #[nestforge::summary("List users")]
    #[nestforge::tag("users")]
    #[nestforge::response(status = 200, description = "Users returned")]
    async fn list() -> nestforge::ApiResult<Vec<String>> {
        Ok(axum::Json(vec!["alice".to_string(), "bob".to_string()]))
    }

    #[nestforge::get("/me")]
    #[nestforge::summary("Get current user")]
    #[nestforge::authenticated]
    #[nestforge::response(status = 200, description = "Current user returned")]
    async fn me() -> nestforge::ApiResult<String> {
        Ok(axum::Json("alice".to_string()))
    }
}

struct AppModule;

impl ModuleDefinition for AppModule {
    fn register(_container: &Container) -> anyhow::Result<()> {
        Ok(())
    }

    fn controllers() -> Vec<axum::Router<Container>> {
        vec![<UsersController as nestforge::ControllerDefinition>::router()]
    }

    fn route_docs() -> Vec<nestforge::RouteDocumentation> {
        <UsersController as nestforge::DocumentedController>::route_docs()
    }
}

#[test]
fn openapi_doc_for_module_collects_documented_routes() {
    let doc = openapi_doc_for_module::<AppModule>("Test API", "1.0.0")
        .expect("openapi doc should generate");

    assert_eq!(doc.routes.len(), 2);
    assert!(doc.routes.iter().any(|route| route.requires_auth));
    assert!(doc
        .routes
        .iter()
        .any(|route| route.required_roles.iter().any(|role| role == "admin")));
    assert!(doc
        .routes
        .iter()
        .any(|route| route.tags.iter().any(|tag| tag == "controller-users")));
    assert!(doc
        .routes
        .iter()
        .any(|route| route.summary.as_deref() == Some("List users")));
}

#[tokio::test]
async fn openapi_docs_router_supports_custom_docs_path_and_yaml_export() {
    let app = openapi_docs_router_for_module_with_config::<AppModule>(
        "Test API",
        "1.0.0",
        OpenApiConfig::new()
            .with_docs_path("/api/docs")
            .with_default_ui(OpenApiUi::SwaggerUi),
    )
    .expect("openapi docs router should build")
    .with_state(Container::new());

    let docs_response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/docs")
                .body(axum::body::Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("docs request should succeed");

    assert_eq!(docs_response.status(), axum::http::StatusCode::OK);

    let yaml_response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/openapi.yaml")
                .body(axum::body::Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("yaml request should succeed");

    assert_eq!(yaml_response.status(), axum::http::StatusCode::OK);
}
