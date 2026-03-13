use nestforge::{injectable, ConfigModule, ConfigOptions};

#[injectable(factory = load_app_config)]
pub struct AppConfig {
    pub app_name: String,
    pub log_level: String,
}

fn load_app_config() -> anyhow::Result<AppConfig> {
    let allowed_levels = vec!["trace", "debug", "info", "warn", "error"];
    let schema = nestforge::EnvSchema::new()
        .min_len("APP_NAME", 2)
        .one_of("LOG_LEVEL", &allowed_levels);

    Ok(ConfigModule::for_root::<AppConfig>(
        ConfigOptions::new().env_file(".env").validate_with(schema),
    )?)
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
