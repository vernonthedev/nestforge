use nestforge::{
    injectable, ConfigError, ConfigModule, ConfigOptions, ConfigService, EnvStore, FromEnv,
};

#[injectable(factory = load_app_config)]
pub struct AppConfig {
    pub app_name: String,
}

fn load_app_config() -> anyhow::Result<AppConfig> {
    let options = ConfigOptions::new().env_file(".env");
    let config = ConfigModule::for_root_with_options(options);
    Ok(AppConfig {
        app_name: config.get_string_or("APP_NAME", "NestForge WebSockets"),
    })
}

impl FromEnv for AppConfig {
    fn from_env(env: &EnvStore) -> Result<Self, ConfigError> {
        Ok(Self {
            app_name: env
                .get("APP_NAME")
                .unwrap_or("NestForge WebSockets")
                .to_string(),
        })
    }
}
