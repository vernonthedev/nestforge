use nestforge::{injectable, ConfigModule, ConfigOptions};

#[injectable(factory = load_app_config)]
pub struct AppConfig {
    pub app_name: String,
}

fn load_app_config() -> anyhow::Result<AppConfig> {
    Ok(ConfigModule::for_root::<AppConfig>(
        ConfigOptions::new().env_file(".env"),
    )?)
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
