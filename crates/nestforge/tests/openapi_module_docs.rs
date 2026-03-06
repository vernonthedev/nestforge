#![cfg(feature = "openapi")]

use nestforge::{
    authenticated, controller, get, openapi_doc_for_module, response, roles, routes, summary, tag,
    Container, ModuleDefinition,
};

#[controller("/users")]
struct UsersController;

#[routes]
#[tag("controller-users")]
#[authenticated]
#[roles("admin")]
impl UsersController {
    #[get("/")]
    #[summary("List users")]
    #[tag("users")]
    #[response(status = 200, description = "Users returned")]
    async fn list() -> nestforge::ApiResult<Vec<String>> {
        Ok(axum::Json(vec!["alice".to_string(), "bob".to_string()]))
    }

    #[get("/me")]
    #[summary("Get current user")]
    #[authenticated]
    #[response(status = 200, description = "Current user returned")]
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
