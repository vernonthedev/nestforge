/**
* This is the entry file for nestforge-core.
* Think of this crate as the framework "brain":
* - DI container
* - module traits
* - controller traits
* - framework error types   
* - DI helpers
*/
pub mod container;
pub mod error;
pub mod module;
pub mod inject;

/**
 * Re-export these so other crates can import from nestforge_core nicely.
 */
pub use container::Container;
pub use error::HttpException;
pub use inject::Inject;
pub use module::{ControllerDefinition, ModuleDefinition};