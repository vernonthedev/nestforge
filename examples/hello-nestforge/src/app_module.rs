use nestforge::{module, Db, DbConfig};

use crate::{
    controllers::{
        AppController, HealthController, UsersController, VersioningController,
    },
    services::{users_service_seed, AppConfig, UsersService},
};

fn load_app_config() -> anyhow::Result<AppConfig> {
    Ok(nestforge::load_config::<AppConfig>()?)
}

fn connect_db() -> anyhow::Result<Db> {
    Ok(Db::connect_lazy(DbConfig::postgres_local("postgres"))?)
}

#[module(
    imports = [],
    controllers = [
        AppController,
        HealthController,
        UsersController,
        VersioningController
    ],
    providers = [
        load_app_config()?,
        users_service_seed(),
        connect_db()?
    ],
    exports = [UsersService, Db]
)]
pub struct AppModule;
