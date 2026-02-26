use nestforge::{module, ConfigModule, ConfigOptions, Db, DbConfig};

use crate::{
    settings_module::SettingsModule,
    controllers::{
        AppController, HealthController, UsersController, VersioningController,
    },
    services::{users_service_seed, AppConfig, UsersService},
};

fn load_app_config() -> anyhow::Result<AppConfig> {
    let schema = nestforge::EnvSchema::new()
        .min_len("APP_NAME", 2)
        .one_of("LOG_LEVEL", &["trace", "debug", "info", "warn", "error"]);

    Ok(ConfigModule::for_root::<AppConfig>(
        ConfigOptions::new().env_file(".env").validate_with(schema),
    )?)
}

fn connect_db() -> anyhow::Result<Db> {
    Ok(Db::connect_lazy(DbConfig::postgres_local("postgres"))?)
}

#[module(
    imports = [SettingsModule],
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
