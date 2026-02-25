use anyhow::Result;

use crate::Container;

/**
* ModuleDefinition = the first Nest-like concept in NestForge.
* 
* For now it only does one job:
* - register providers/services into the container at startup.
* 
* Later this trait will grow to also include:
* - controllers
* - imported modules
* - exported providers
*/
pub trait ModuleDefinition: Send + Sync + 'static {
    /**
    * Called by the factory when the app starts.
    * 
    * The module gets access to the DI container and can register what it needs.
    */
    fn register(container: &Container) -> Result<()>;
}