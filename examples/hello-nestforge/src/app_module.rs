use nestforge::{module, Db, DbConfig, Provider};

use crate::{
    controllers::{AppController, HealthController, UsersController},
    services::{AppConfig, UsersService},
};
#[module(
    imports = [],
    controllers = [AppController, HealthController, UsersController],
    providers = [
        Provider::value(nestforge::load_config::<AppConfig>()?),
        Provider::factory(|_| Ok(UsersService::new())),
        Provider::value(Db::connect_lazy(DbConfig::postgres_local("postgres"))?)
    ],
    exports = [UsersService, Db]
)]
pub struct AppModule;
