pub mod controllers;
pub mod dto;
pub mod services;

use nestforge::module;

use self::controllers::SettingsController;
use self::services::{SettingsService, settings_service_seed};

#[module(
    imports = [],
    controllers = [SettingsController],
    providers = [settings_service_seed()],
    exports = [SettingsService]
)]
pub struct SettingsModule;
