use nestforge::{controller, routes, HttpException, Inject};

use crate::services::AppConfig;

#[controller("")]
pub struct AppController;

#[routes]
impl AppController {
    #[get("/")]
    async fn root(cfg: Inject<AppConfig>) -> Result<String, HttpException> {
        Ok(format!("Welcome to {} 🔥", cfg.app_name))
    }
}