/**
* This is the crate users will import.
* 
* It re-exports the internal pieces so the public API feels clean:
* use nestforge::{NestForgeFactory, ModuleDefinition, Container};
*/

pub use nestforge_core::{Container, ControllerDefinition, ModuleDefinition};
pub use nestforge_http::NestForgeFactory;