use nestforge::module;

use crate::app_config::AppConfig;
use crate::grpc::GrpcPatterns;

#[module(
    imports = [],
    controllers = [],
    providers = [AppConfig, GrpcPatterns],
    exports = [AppConfig, GrpcPatterns]
)]
pub struct AppModule;
