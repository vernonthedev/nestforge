use nestforge::prelude::*;

use crate::{AppConfig, GrpcPatterns};

#[module(
    imports = [],
    controllers = [],
    providers = [AppConfig, GrpcPatterns],
    exports = [AppConfig, GrpcPatterns]
)]
pub struct AppModule;
