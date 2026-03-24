use nestforge::{module, ConfigService, Db, DbConfig};

use crate::{
    app_config::load_config, settings::SettingsModule, users::UsersModule,
    versioning::VersioningModule, AppConfig, AppController, HealthController,
};

fn connect_db() -> anyhow::Result<Db> {
    Ok(Db::connect_lazy(DbConfig::postgres_local("postgres"))?)
}

#[module(
    imports = [UsersModule, SettingsModule, VersioningModule],
    controllers = [AppController, HealthController],
    providers = [
        AppConfig,
        load_config(),
        connect_db()?
    ],
    exports = [Db, AppConfig, ConfigService]
)]
pub struct AppModule;
