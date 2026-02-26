use nestforge::{module, Db, DbConfig, Provider};

use crate::{
    controllers::{AppController, HealthController, UsersController},
    services::{users_service_seed, AppConfig, UsersService},
};
#[module(
    imports = [],
    controllers = [AppController, HealthController, UsersController],
    providers = [
        Provider::value(nestforge::load_config::<AppConfig>()?),
        Provider::factory(|_| Ok(users_service_seed())),
        Provider::value(Db::connect_lazy(DbConfig::postgres_local("postgres"))?)
    ],
    exports = [UsersService, Db]
)]
pub struct AppModule;
