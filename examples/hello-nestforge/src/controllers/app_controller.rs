use axum::{extract::State, routing::get, Router};
use nestforge::{Container, ControllerDefinition};

use crate::services::AppConfig;

/*
AppController = handles "/"
*/
pub struct AppController;

impl ControllerDefinition for AppController {
    fn router() -> Router<Container> {
        Router::new().route("/", get(Self::root))
    }
}

impl AppController {
    /*
    Pull AppConfig from DI and build the welcome message
    */
    async fn root(State(container): State<Container>) -> String {
        match container.resolve::<AppConfig>() {
            Ok(cfg) => format!("Welcome to {} 🔥", cfg.app_name),
            Err(_) => "Welcome to NestForge 🔥".to_string(),
        }
    }
}