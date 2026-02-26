use nestforge::{controller, routes, Db, HttpException, Inject};

#[controller("")]
pub struct HealthController;

#[routes]
impl HealthController {
    #[get("/health")]
    async fn health() -> &'static str {
        "OK"
    }

    #[get("/health/db")]
    async fn health_db(_db: Inject<Db>) -> Result<&'static str, HttpException> {
        Ok("DB_READY")
    }
}
