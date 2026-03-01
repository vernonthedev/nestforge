use nestforge::{module, ConfigModule, ConfigOptions};

use crate::app_config::AppConfig;
use crate::ws::WsPatterns;

fn load_app_config() -> anyhow::Result<AppConfig> {
    Ok(ConfigModule::for_root::<AppConfig>(
        ConfigOptions::new().env_file(".env"),
    )?)
}

fn load_ws_patterns() -> anyhow::Result<WsPatterns> {
    Ok(WsPatterns::new())
}

#[module(
    imports = [],
    controllers = [],
    providers = [load_app_config()?, load_ws_patterns()?],
    exports = [AppConfig, WsPatterns]
)]
pub struct AppModule;
