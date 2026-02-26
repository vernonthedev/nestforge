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
    Guard,
    HttpException,
    Inject,
    Provider,
    RegisterProvider,
    ModuleRef,
    Interceptor,
    ModuleDefinition,
    NextFn,
    NextFuture,
    Param,
    RequestContext,
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
pub use nestforge_macros::{
    controller, entity, get, id, module, post, put, routes, use_guard, use_interceptor,
};
#[cfg(feature = "testing")]
pub use nestforge_testing::{TestFactory, TestingModule};
#[cfg(feature = "db")]
pub use nestforge_db::{Db, DbConfig, DbError, DbTransaction};
#[cfg(feature = "orm")]
pub use nestforge_orm::{EntityMeta, OrmError, Repo, RepoFuture, SqlRepo, SqlRepoBuilder};
#[cfg(feature = "config")]
pub use nestforge_config::{load_config, ConfigError, EnvStore, FromEnv};
#[cfg(feature = "openapi")]
pub use nestforge_openapi::{docs_router, OpenApiDoc, OpenApiRoute};
#[cfg(feature = "data")]
pub use nestforge_data::{CacheStore, DataError, DataFuture, DocumentRepo};
#[cfg(feature = "mongo")]
pub use nestforge_mongo::{InMemoryMongoRepo, MongoConfig};
#[cfg(feature = "redis")]
pub use nestforge_redis::{InMemoryRedisStore, RedisConfig};
