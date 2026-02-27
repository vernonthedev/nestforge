pub mod controllers;

use nestforge::module;

use self::controllers::VersioningController;

#[module(
    imports = [],
    controllers = [VersioningController],
    providers = [],
    exports = []
)]
pub struct VersioningModule;
