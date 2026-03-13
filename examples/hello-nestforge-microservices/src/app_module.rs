use nestforge::prelude::*;

use crate::{AppConfig, AppPatterns, EventCounter};

#[module(
    imports = [],
    controllers = [],
    providers = [AppConfig, AppPatterns, EventCounter],
    exports = [AppConfig, AppPatterns, EventCounter]
)]
pub struct AppModule;
