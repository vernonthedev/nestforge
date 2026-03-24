use nestforge::{injectable, ConfigError, ConfigOptions, ConfigService, EnvStore, FromEnv};

#[injectable(factory = load_app_config)]
pub struct AppConfig {
    pub app_name: String,
    pub log_level: String,
}

impl FromEnv for AppConfig {
    fn from_env(env: &EnvStore) -> Result<Self, ConfigError> {
        Ok(AppConfig {
            app_name: env.get("APP_NAME").unwrap_or("NestForge").to_string(),
            log_level: env.get("LOG_LEVEL").unwrap_or("info").to_string(),
        })
    }
}

fn load_app_config() -> anyhow::Result<AppConfig> {
    let options = ConfigOptions::new().env_file(".env");
    let config = ConfigService::load_with_options(&options)?;
    let env_store = EnvStore::from(config);
    AppConfig::from_env(&env_store).map_err(Into::into)
}

pub fn load_config() -> ConfigService {
    ConfigService::load_with_options(&ConfigOptions::new().env_file(".env")).unwrap()
}
