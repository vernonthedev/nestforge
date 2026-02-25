/*
AppConfig = tiny config object stored in DI.
Later this could become a proper config service.
*/

#[derive(Clone)]
pub struct AppConfig {
    pub app_name: String,
}