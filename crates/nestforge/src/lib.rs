/**
* This is the crate users will import.
* It re-exports the internal pieces
* use nestforge::{NestForgeFactory, ModuleDefinition, Container};
*/
pub use nestforge_core::{
    Body,
    Container,
    ControllerBasePath,
    ControllerDefinition,
    HttpException,
    Inject,
    ModuleDefinition,
    Param,
    RouteBuilder,
};

pub use nestforge_http::NestForgeFactory;
pub use nestforge_macros::{controller, get, post, put, routes};