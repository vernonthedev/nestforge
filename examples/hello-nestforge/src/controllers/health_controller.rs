use axum::{routing::get, Router};
use nestforge::{Container, ControllerDefinition};

/*
HealthController = handles "/health"
*/
pub struct HealthController;

impl ControllerDefinition for HealthController {
    fn router() -> Router<Container> {
        Router::new().route("/health", get(Self::health))
    }
}

impl HealthController {
    async fn health() -> &'static str {
        "OK"
    }
}