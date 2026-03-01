use std::sync::{atomic::AtomicUsize, Arc};

use nestforge::{module, ConfigModule, ConfigOptions};

use crate::{
    app_config::AppConfig,
    microservices::{AppPatterns, EventCounter},
};

fn load_app_config() -> anyhow::Result<AppConfig> {
    let schema = nestforge::EnvSchema::new().min_len("APP_NAME", 2);

    Ok(ConfigModule::for_root::<AppConfig>(
        ConfigOptions::new().env_file(".env").validate_with(schema),
    )?)
}

fn load_patterns() -> anyhow::Result<AppPatterns> {
    Ok(AppPatterns::new())
}

fn load_event_counter() -> anyhow::Result<EventCounter> {
    Ok(EventCounter(Arc::new(AtomicUsize::new(0))))
}

#[module(
    imports = [],
    controllers = [],
    providers = [load_app_config()?, load_patterns()?, load_event_counter()?],
    exports = [AppConfig, AppPatterns, EventCounter]
)]
pub struct AppModule;
