use nestforge::prelude::*;

use crate::{AppConfig, WsPatterns};

#[module(
    imports = [],
    controllers = [],
    providers = [AppConfig, WsPatterns],
    exports = [AppConfig, WsPatterns]
)]
pub struct AppModule;
