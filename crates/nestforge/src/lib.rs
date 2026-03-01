/**
* This is the crate users will import.
* It re-exports the internal pieces
* use nestforge::{NestForgeFactory, ModuleDefinition, Container};
*/
pub use nestforge_core::{
    collect_module_route_docs, initialize_module_graph, register_provider, ApiResult,
    AuthIdentity, AuthUser, BearerToken, Body, Container, ContainerError, ControllerBasePath,
    ControllerDefinition, DocumentedController, Guard, HttpException, Identifiable, InMemoryStore,
    Inject, Interceptor, List, ModuleDefinition, ModuleRef, NextFn, NextFuture, OptionHttpExt,
    Param, Provider, Query, RegisterProvider, RequestContext, RequestId, ResourceError,
    ResourceService, ResultHttpExt, RouteBuilder, RouteDocumentation,
    RouteResponseDocumentation, Validate, ValidatedBody, ValidationErrors, ValidationIssue,
    framework_log, framework_log_event,
};

#[cfg(feature = "config")]
pub use nestforge_config::{
    load_config, ConfigError, ConfigModule, ConfigOptions, EnvSchema, EnvStore, EnvValidationIssue,
    FromEnv,
};
#[cfg(feature = "data")]
pub use nestforge_data::{CacheStore, DataError, DataFuture, DocumentRepo};
#[cfg(feature = "db")]
pub use nestforge_db::{Db, DbConfig, DbError, DbTransaction};
pub use nestforge_http::NestForgeFactory;
pub use nestforge_macros::{
    authenticated, controller, delete, description, dto, entity, entity_dto, get, id,
    identifiable, module, post, put, response, response_dto, routes, summary, tag, use_guard,
    use_interceptor, version, Identifiable, Validate,
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

#[macro_export]
macro_rules! guard {
    ($name:ident) => {
        #[derive(Default)]
        pub struct $name;

        impl $crate::Guard for $name {
            fn can_activate(
                &self,
                _ctx: &$crate::RequestContext,
            ) -> Result<(), $crate::HttpException> {
                Ok(())
            }
        }
    };
    ($name:ident, |$ctx:ident| $body:block) => {
        #[derive(Default)]
        pub struct $name;

        impl $crate::Guard for $name {
            fn can_activate(
                &self,
                $ctx: &$crate::RequestContext,
            ) -> Result<(), $crate::HttpException> {
                $body
            }
        }
    };
}

#[macro_export]
macro_rules! interceptor {
    ($name:ident) => {
        #[derive(Default)]
        pub struct $name;

        impl $crate::Interceptor for $name {
            fn around(
                &self,
                _ctx: $crate::RequestContext,
                req: axum::extract::Request,
                next: $crate::NextFn,
            ) -> $crate::NextFuture {
                Box::pin(async move { (next)(req).await })
            }
        }
    };
    ($name:ident, |$ctx:ident, $req:ident, $next:ident| $body:block) => {
        #[derive(Default)]
        pub struct $name;

        impl $crate::Interceptor for $name {
            fn around(
                &self,
                $ctx: $crate::RequestContext,
                $req: axum::extract::Request,
                $next: $crate::NextFn,
            ) -> $crate::NextFuture {
                Box::pin(async move $body)
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

#[cfg(feature = "openapi")]
pub fn openapi_doc_for_module<M: ModuleDefinition>(
    title: impl Into<String>,
    version: impl Into<String>,
) -> anyhow::Result<OpenApiDoc> {
    let routes = collect_module_route_docs::<M>()?;
    Ok(OpenApiDoc::from_routes(title, version, routes))
}

#[cfg(feature = "openapi")]
pub fn openapi_docs_router_for_module<M: ModuleDefinition>(
    title: impl Into<String>,
    version: impl Into<String>,
) -> anyhow::Result<axum::Router> {
    let doc = openapi_doc_for_module::<M>(title, version)?;
    Ok(docs_router(doc))
}
