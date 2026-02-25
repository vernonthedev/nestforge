/**
* This is the entry file for nestforge-core.
* 
* Think of this crate as the framework "brain":
* - DI container
* - module traits
* - (later) controller/provider metadata
*/

pub mod container;
pub mod module;

/**
 * Re-export these so other crates can import from nestforge_core nicely.
 */
pub use container::Container;
pub use module::ModuleDefinition;