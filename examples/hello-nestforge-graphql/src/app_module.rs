use nestforge::module;

use crate::app_config::AppConfig;

#[module(
    imports = [],
    controllers = [],
    providers = [AppConfig],
    exports = [AppConfig]
)]
pub struct AppModule;
