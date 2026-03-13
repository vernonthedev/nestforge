use nestforge::prelude::*;

use crate::AppConfig;
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
