use anyhow::Result;
use axum::Router;

use crate::Container;

/**
* ControllerDefinition = manual controller contract (pre-macros).
* 
* Each controller returns an axum Router<Container> with its routes.
* Later our macros will generate this automatically.
*/
pub trait ControllerDefinition: Send + Sync + 'static {
    fn router() -> Router<Container>;
}

/**
* ModuleDefinition = first Nest-like concept in NestForge.
* 
* Now it does two jobs:
* 1) register providers/services into DI
* 2) return controller routers to be mounted by the factory.
*/
pub trait ModuleDefinition: Send + Sync + 'static {
    fn register(container: &Container) -> Result<()>;

    /*
    Controllers exposed by this module.
    For now it's just a Vec of routers (manual style).
    */
    fn controllers() -> Vec<Router<Container>> {
        Vec::new()
    }
}