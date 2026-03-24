use nestforge::{injectable, ConfigModule, ConfigOptions, ConfigService};

#[injectable(factory = load_app_config)]
pub struct AppConfig {
    pub app_name: String,
}

fn load_app_config() -> anyhow::Result<AppConfig> {
    let options = ConfigOptions::new().env_file(".env");
    let config = ConfigModule::try_for_root_with_options(options)?;
    Ok(AppConfig {
        app_name: config.get_string_or("APP_NAME", "NestForge gRPC"),
    })
}

pub fn load_config() -> ConfigService {
    ConfigModule::for_root_with_options(ConfigOptions::new().env_file(".env"))
}
