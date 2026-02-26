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
    Provider,
    RegisterProvider,
    ModuleRef,
    ModuleDefinition,
    Param,
    RouteBuilder,
    register_provider,
    initialize_module_graph,
    Identifiable,
    InMemoryStore,
};

pub use nestforge_http::NestForgeFactory;
pub use nestforge_macros::{controller, get, post, put, routes, module};
