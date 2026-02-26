use nestforge::{module, Container, Provider};

use crate::{
    controllers::SettingsController,
    services::{AppConfig, SettingsService, settings_service_from_config},
};

fn build_settings_service(container: &Container) -> anyhow::Result<SettingsService> {
    let app_config = container.resolve::<AppConfig>()?;
    Ok(settings_service_from_config(app_config.as_ref()))
}

#[module(
    imports = [],
    controllers = [SettingsController],
    providers = [Provider::factory(build_settings_service)],
    exports = [SettingsService]
)]
pub struct SettingsModule;
