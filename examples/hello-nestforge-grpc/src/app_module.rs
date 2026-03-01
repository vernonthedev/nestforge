use nestforge::{module, ConfigModule, ConfigOptions};

use crate::app_config::AppConfig;

fn load_app_config() -> anyhow::Result<AppConfig> {
    let schema = nestforge::EnvSchema::new().min_len("APP_NAME", 2);

    Ok(ConfigModule::for_root::<AppConfig>(
        ConfigOptions::new().env_file(".env").validate_with(schema),
    )?)
}

#[module(
    imports = [],
    controllers = [],
    providers = [load_app_config()?],
    exports = [AppConfig]
)]
pub struct AppModule;
