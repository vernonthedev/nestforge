use nestforge::{controller, routes, ApiSerializedResult, HttpException, Inject, Serialized};

use crate::app_config::AppConfig;
use crate::serializers::AppConfigSerializer;

#[controller("")]
pub struct AppController;

#[routes]
impl AppController {
    #[nestforge::get("/")]
    async fn root(cfg: Inject<AppConfig>) -> Result<String, HttpException> {
        Ok(format!("Welcome to {}", cfg.app_name))
    }

    #[nestforge::get("/info")]
    async fn info(cfg: Inject<AppConfig>) -> ApiSerializedResult<AppConfig, AppConfigSerializer> {
        Ok(Serialized::new((*cfg).clone()))
    }
}
