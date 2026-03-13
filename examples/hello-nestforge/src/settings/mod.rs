pub mod controllers;
pub mod dto;
pub mod services;

use nestforge::module;

use self::controllers::SettingsController;
use self::services::SettingsService;

#[module(
    imports = [],
    controllers = [SettingsController],
    providers = [SettingsService],
    exports = [SettingsService]
)]
pub struct SettingsModule;
