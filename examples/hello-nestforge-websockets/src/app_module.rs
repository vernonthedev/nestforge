use nestforge::module;

use crate::app_config::AppConfig;
use crate::ws::WsPatterns;

#[module(
    imports = [],
    controllers = [],
    providers = [AppConfig, WsPatterns],
    exports = [AppConfig, WsPatterns]
)]
pub struct AppModule;
