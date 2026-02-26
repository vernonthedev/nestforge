use nestforge::{controller, routes};

#[controller("/versioning")]
pub struct VersioningController;

#[routes]
impl VersioningController {
    #[nestforge::get("/hello")]
    #[nestforge::version("1")]
    async fn hello_v1() -> String {
        "Hello from API v1".to_string()
    }

    #[nestforge::get("/hello")]
    #[nestforge::version("2")]
    async fn hello_v2() -> String {
        "Hello from API v2".to_string()
    }
}
