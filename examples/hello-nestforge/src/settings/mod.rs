pub mod controllers;
pub mod dto;
pub mod services;

use nestforge::module;

use self::controllers::SettingsController;
use self::services::{settings_service_seed, SettingsService};

#[module(
    imports = [],
    controllers = [SettingsController],
    providers = [settings_service_seed()],
    exports = [SettingsService]
)]
pub struct SettingsModule;
