use nestforge::{module, ConfigModule, ConfigOptions};

use crate::app_config::AppConfig;

fn load_app_config() -> anyhow::Result<AppConfig> {
    Ok(ConfigModule::for_root::<AppConfig>(
        ConfigOptions::new().env_file(".env"),
    )?)
}

#[module(
    imports = [],
    controllers = [],
    providers = [load_app_config()?],
    exports = [AppConfig]
)]
pub struct AppModule;
