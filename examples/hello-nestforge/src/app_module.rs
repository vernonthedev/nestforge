use nestforge::{module, ConfigModule, ConfigOptions, Db, DbConfig};

use crate::{
    app_config::AppConfig,
    app_controller::AppController,
    health_controller::HealthController,
    settings::SettingsModule,
    users::UsersModule,
    versioning::VersioningModule,
};

fn load_app_config() -> anyhow::Result<AppConfig> {
    let allowed_levels = vec!["trace", "debug", "info", "warn", "error"];
    let schema = nestforge::EnvSchema::new()
        .min_len("APP_NAME", 2)
        .one_of("LOG_LEVEL", &allowed_levels);

    Ok(ConfigModule::for_root::<AppConfig>(
        ConfigOptions::new().env_file(".env").validate_with(schema),
    )?)
}

fn connect_db() -> anyhow::Result<Db> {
    Ok(Db::connect_lazy(DbConfig::postgres_local("postgres"))?)
}

#[module(
    imports = [UsersModule, SettingsModule, VersioningModule],
    controllers = [AppController, HealthController],
    providers = [
        load_app_config()?,
        connect_db()?
    ],
    exports = [Db, AppConfig]
)]
pub struct AppModule;
