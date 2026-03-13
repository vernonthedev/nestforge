use nestforge::{module, Db, DbConfig};

use crate::{
    app_config::AppConfig, app_controller::AppController, health_controller::HealthController,
    settings::SettingsModule, users::UsersModule, versioning::VersioningModule,
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
