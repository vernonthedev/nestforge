/**
* This is the crate users will import.
* It re-exports the internal pieces
* use nestforge::{NestForgeFactory, ModuleDefinition, Container};
*/
pub use nestforge_core::{
    Body,
    Container,
    ContainerError,
    ControllerBasePath,
    ControllerDefinition,
    HttpException,
    Inject,
    Provider,
    RegisterProvider,
    ModuleRef,
    ModuleDefinition,
    Param,
    Validate,
    ValidatedBody,
    ValidationErrors,
    ValidationIssue,
    RouteBuilder,
    register_provider,
    initialize_module_graph,
    Identifiable,
    InMemoryStore,
};

pub use nestforge_http::NestForgeFactory;
pub use nestforge_macros::{controller, entity, get, id, module, post, put, routes};
#[cfg(feature = "testing")]
pub use nestforge_testing::{TestFactory, TestingModule};
#[cfg(feature = "db")]
pub use nestforge_db::{Db, DbConfig, DbError, DbTransaction};
#[cfg(feature = "orm")]
pub use nestforge_orm::{EntityMeta, OrmError, Repo, RepoFuture, SqlRepo, SqlRepoBuilder};
