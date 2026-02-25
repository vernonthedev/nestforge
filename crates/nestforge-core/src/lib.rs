/**
* This is the entry file for nestforge-core.
* Think of this crate as the framework "brain":
* - DI container
* - module traits
* - controller traits
* - framework error types   
*/
pub mod container;
pub mod error;
pub mod module;

/**
 * Re-export these so other crates can import from nestforge_core nicely.
 */
pub use container::Container;
pub use error::HttpException;
pub use module::{ControllerDefinition, ModuleDefinition};