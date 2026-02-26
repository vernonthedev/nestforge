use nestforge::{controller, routes, Db, HttpException, Inject};

#[controller("")]
pub struct HealthController;

#[routes]
impl HealthController {
    #[nestforge::get("/health")]
    async fn health() -> String {
        "OK".to_string()
    }

    #[nestforge::get("/health/db")]
    async fn health_db(_db: Inject<Db>) -> Result<String, HttpException> {
        Ok("DB_READY".to_string())
    }
}
