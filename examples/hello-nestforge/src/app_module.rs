use nestforge::module;

use crate::{
    controllers::{AppController, HealthController, UsersController},
    services::{AppConfig, UsersService},
};
#[module(
    controllers = [AppController, HealthController, UsersController],
    providers = [
        AppConfig { app_name: "NestForge".to_string() },
        UsersService::new()
    ]
)]
pub struct AppModule;