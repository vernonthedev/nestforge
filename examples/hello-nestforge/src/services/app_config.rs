/*
AppConfig = tiny config object stored in DI.
Later this could become a proper config service.
*/

#[derive(Clone)]
pub struct AppConfig {
    pub app_name: String,
    pub log_level: String,
}

impl nestforge::FromEnv for AppConfig {
    fn from_env(env: &nestforge::EnvStore) -> Result<Self, nestforge::ConfigError> {
        let app_name = env.get("APP_NAME").unwrap_or("NestForge").to_string();
        let log_level = env.get("LOG_LEVEL").unwrap_or("info").to_string();
        Ok(Self {
            app_name,
            log_level,
        })
    }
}
