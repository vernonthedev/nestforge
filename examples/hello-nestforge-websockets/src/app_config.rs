#[derive(Debug, Clone)]
pub struct AppConfig {
    pub app_name: String,
}

impl nestforge::FromEnv for AppConfig {
    fn from_env(env: &nestforge::EnvStore) -> Result<Self, nestforge::ConfigError> {
        let app_name = env
            .get("APP_NAME")
            .unwrap_or("NestForge WebSockets")
            .to_string();
        Ok(Self { app_name })
    }
}
