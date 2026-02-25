use axum::{extract::State, routing::get, Router};
use nestforge::{Container, ControllerDefinition, HttpException};

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
    Pull AppConfig from DI and build the welcome message.
    If config is missing, return a proper framework error.
    */
    async fn root(State(container): State<Container>) -> Result<String, HttpException> {
        let cfg = container.resolve::<AppConfig>().map_err(|_| {
            HttpException::internal_server_error("[AppController] AppConfig is not registered in the container")
        })?;

        Ok(format!("Welcome to {}", cfg.app_name))
    }
}