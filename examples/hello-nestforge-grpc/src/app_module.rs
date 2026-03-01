use nestforge::{module, ConfigModule, ConfigOptions};

use crate::app_config::AppConfig;
use crate::grpc::GrpcPatterns;

fn load_app_config() -> anyhow::Result<AppConfig> {
    let schema = nestforge::EnvSchema::new().min_len("APP_NAME", 2);

    Ok(ConfigModule::for_root::<AppConfig>(
        ConfigOptions::new().env_file(".env").validate_with(schema),
    )?)
}

fn load_grpc_patterns() -> anyhow::Result<GrpcPatterns> {
    Ok(GrpcPatterns::new())
}

#[module(
    imports = [],
    controllers = [],
    providers = [load_app_config()?, load_grpc_patterns()?],
    exports = [AppConfig, GrpcPatterns]
)]
pub struct AppModule;
