use nestforge::prelude::*;

use crate::AppConfig;

#[module(
    imports = [],
    controllers = [],
    providers = [AppConfig],
    exports = [AppConfig]
)]
pub struct AppModule;
