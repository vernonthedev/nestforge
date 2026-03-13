pub mod controllers;
pub mod dto;
pub mod services;

use nestforge::module;

use self::controllers::UsersController;
use self::services::UsersService;

#[module(
    imports = [],
    controllers = [UsersController],
    providers = [UsersService],
    exports = [UsersService]
)]
pub struct UsersModule;
