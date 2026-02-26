use nestforge::{module, Provider};

use crate::{
    controllers::{AppController, HealthController, UsersController},
    services::{AppConfig, UsersService},
};
#[module(
    imports = [],
    controllers = [AppController, HealthController, UsersController],
    providers = [
        Provider::value(AppConfig { app_name: "NestForge".to_string() }),
        Provider::factory(|_| Ok(UsersService::new()))
    ],
    exports = [UsersService]
)]
pub struct AppModule;
