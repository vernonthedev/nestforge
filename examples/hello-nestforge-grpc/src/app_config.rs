use nestforge::{injectable, ConfigModule, ConfigOptions};

#[injectable(factory = load_app_config)]
pub struct AppConfig {
    pub app_name: String,
}

fn load_app_config() -> anyhow::Result<AppConfig> {
    let schema = nestforge::EnvSchema::new().min_len("APP_NAME", 2);

    Ok(ConfigModule::for_root::<AppConfig>(
        ConfigOptions::new().env_file(".env").validate_with(schema),
    )?)
}

impl nestforge::FromEnv for AppConfig {
    fn from_env(env: &nestforge::EnvStore) -> Result<Self, nestforge::ConfigError> {
        Ok(Self {
            app_name: env.get("APP_NAME").unwrap_or("NestForge gRPC").to_string(),
        })
    }
}
