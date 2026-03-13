use nestforge::{module, Db, DbConfig};

use crate::{
    AppConfig, AppController, HealthController, settings::SettingsModule, users::UsersModule,
    versioning::VersioningModule,
};

fn connect_db() -> anyhow::Result<Db> {
    Ok(Db::connect_lazy(DbConfig::postgres_local("postgres"))?)
}

#[module(
    imports = [UsersModule, SettingsModule, VersioningModule],
    controllers = [AppController, HealthController],
    providers = [
        AppConfig,
        connect_db()?
    ],
    exports = [Db, AppConfig]
)]
pub struct AppModule;
