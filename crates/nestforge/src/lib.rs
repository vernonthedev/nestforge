/**
* This is the crate users will import.
* It re-exports the internal pieces
* use nestforge::{NestForgeFactory, ModuleDefinition, Container};
*/
pub use nestforge_core::{
    initialize_module_graph, register_provider, ApiResult, Body, Container, ContainerError,
    ControllerBasePath, ControllerDefinition, Guard, HttpException, Identifiable, InMemoryStore,
    Inject, Interceptor, List, ModuleDefinition, ModuleRef, NextFn, NextFuture, Param, Provider,
    RegisterProvider, RequestContext, ResourceError, ResourceService, RouteBuilder, Validate,
    ValidatedBody, ValidationErrors, ValidationIssue,
};

#[cfg(feature = "config")]
pub use nestforge_config::{load_config, ConfigError, EnvStore, FromEnv};
#[cfg(feature = "data")]
pub use nestforge_data::{CacheStore, DataError, DataFuture, DocumentRepo};
#[cfg(feature = "db")]
pub use nestforge_db::{Db, DbConfig, DbError, DbTransaction};
pub use nestforge_http::NestForgeFactory;
pub use nestforge_macros::{
    controller, delete, dto, entity, entity_dto, get, id, identifiable, module, post, put,
    response_dto, routes, use_guard, use_interceptor, Identifiable, Validate,
};

#[macro_export]
macro_rules! impl_identifiable {
    ($type:ty, $field:ident) => {
        impl $crate::Identifiable for $type {
            fn id(&self) -> u64 {
                self.$field
            }

            fn set_id(&mut self, id: u64) {
                self.$field = id;
            }
        }
    };
}
#[cfg(feature = "mongo")]
pub use nestforge_mongo::{InMemoryMongoRepo, MongoConfig};
#[cfg(feature = "openapi")]
pub use nestforge_openapi::{docs_router, OpenApiDoc, OpenApiRoute};
#[cfg(feature = "orm")]
pub use nestforge_orm::{EntityMeta, OrmError, Repo, RepoFuture, SqlRepo, SqlRepoBuilder};
#[cfg(feature = "redis")]
pub use nestforge_redis::{InMemoryRedisStore, RedisConfig};
#[cfg(feature = "testing")]
pub use nestforge_testing::{TestFactory, TestingModule};
