pub mod controllers;
pub mod dto;
pub mod services;

use nestforge::module;

use self::controllers::UsersController;
use self::services::{users_service_seed, UsersService};

#[module(
    imports = [],
    controllers = [UsersController],
    providers = [users_service_seed()],
    exports = [UsersService]
)]
pub struct UsersModule;
