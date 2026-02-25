use nestforge::{controller, routes};

#[controller("")]
pub struct HealthController;

#[routes]
impl HealthController {
    #[get("/health")]
    async fn health() -> &'static str {
        "OK"
    }
}