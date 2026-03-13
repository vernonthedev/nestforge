use nestforge::module;

use crate::{
    app_config::AppConfig,
    microservices::{AppPatterns, EventCounter},
};

#[module(
    imports = [],
    controllers = [],
    providers = [AppConfig, AppPatterns, EventCounter],
    exports = [AppConfig, AppPatterns, EventCounter]
)]
pub struct AppModule;
