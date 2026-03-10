pub struct AppConfig {
    pub app_name: String,
}

impl nestforge::FromEnv for AppConfig {
    fn from_env(env: &nestforge::EnvStore) -> Result<Self, nestforge::ConfigError> {
        Ok(Self {
            app_name: env
                .get("APP_NAME")
                .unwrap_or("NestForge GraphQL")
                .to_string(),
        })
    }
}
