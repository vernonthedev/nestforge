/**
* This is the crate users will import.
* It re-exports the internal pieces
* use nestforge::{NestForgeFactory, ModuleDefinition, Container};
*/
pub use nestforge_core::{
    collect_module_graph, collect_module_route_docs, initialize_module_graph, register_provider,
    ApiEnvelopeResult, ApiResult, AuthIdentity, AuthUser, BearerToken, Body, Container,
    ContainerError, ControllerBasePath, ControllerDefinition, Cookies, Decorated,
    DocumentedController, DynamicModuleBuilder, ExceptionFilter, Guard, Headers, HttpException,
    Identifiable, InMemoryStore, Inject, InitializedModule, Interceptor, LifecycleHook, List,
    ModuleDefinition, ModuleGraphEntry, ModuleGraphReport, ModuleRef, NextFn, NextFuture,
    OptionHttpExt, OptionalAuthUser, Param, Provider, Query, RegisterProvider, RequestContext,
    RequestDecorator, RequestId, RequireAuthenticationGuard, ResourceError, ResourceService,
    ResponseEnvelope, ResultHttpExt, RoleRequirementsGuard, RouteBuilder, RouteDocumentation,
    RouteResponseDocumentation, Validate, ValidatedBody, ValidationErrors, ValidationIssue,
    framework_log, framework_log_event, Pipe, PipedBody, PipedParam, PipedQuery,
};

#[cfg(feature = "config")]
pub use nestforge_config::{
    load_config, ConfigError, ConfigModule, ConfigOptions, EnvSchema, EnvStore, EnvValidationIssue,
    FromEnv,
};
#[cfg(feature = "data")]
pub use nestforge_data::{CacheStore, DataError, DataFuture, DocumentRepo};
#[cfg(feature = "cache")]
pub use nestforge_cache::{cached_response_interceptor, CacheInterceptor, CachePolicy, DefaultCachePolicy};
#[cfg(feature = "microservices")]
pub use nestforge_microservices::{
    EventEnvelope, MessageEnvelope, MicroserviceContext, MicroserviceRegistry,
    MicroserviceRegistryBuilder, TransportMetadata,
};
#[cfg(feature = "db")]
pub use nestforge_db::{Db, DbConfig, DbError, DbTransaction};
pub use nestforge_http::NestForgeFactory;
pub use nestforge_http::{MiddlewareConsumer, MiddlewareRoute, NestMiddleware};
pub use nestforge_macros::{
    authenticated, controller, delete, description, dto, entity, entity_dto, get, id,
    identifiable, module, post, put, response, response_dto, roles, routes, summary, tag,
    use_exception_filter, use_guard, use_interceptor, version, Identifiable, Validate,
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
macro_rules! auth_guard {
    ($name:ident) => {
        #[derive(Default)]
        pub struct $name;

        impl $crate::Guard for $name {
            fn can_activate(
                &self,
                ctx: &$crate::RequestContext,
            ) -> Result<(), $crate::HttpException> {
                if ctx.is_authenticated() {
                    Ok(())
                } else {
                    Err($crate::HttpException::unauthorized("Authentication required"))
                }
            }
        }
    };
}

#[macro_export]
macro_rules! role_guard {
    ($name:ident, $role:expr) => {
        #[derive(Default)]
        pub struct $name;

        impl $crate::Guard for $name {
            fn can_activate(
                &self,
                ctx: &$crate::RequestContext,
            ) -> Result<(), $crate::HttpException> {
                if !ctx.is_authenticated() {
                    return Err($crate::HttpException::unauthorized("Authentication required"));
                }

                if ctx.has_role($role) {
                    Ok(())
                } else {
                    Err($crate::HttpException::forbidden(format!(
                        "Missing required role `{}`",
                        $role
                    )))
                }
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

#[macro_export]
macro_rules! middleware {
    ($name:ident) => {
        #[derive(Default)]
        pub struct $name;

        impl $crate::NestMiddleware for $name {
            fn handle(
                &self,
                req: axum::extract::Request<axum::body::Body>,
                next: $crate::NextFn,
            ) -> $crate::NextFuture {
                Box::pin(async move { (next)(req).await })
            }
        }
    };
    ($name:ident, |$req:ident, $next:ident| $body:block) => {
        #[derive(Default)]
        pub struct $name;

        impl $crate::NestMiddleware for $name {
            fn handle(
                &self,
                $req: axum::extract::Request<axum::body::Body>,
                $next: $crate::NextFn,
            ) -> $crate::NextFuture {
                Box::pin(async move $body)
            }
        }
    };
}

#[macro_export]
macro_rules! request_decorator {
    ($name:ident => $output:ty, |$ctx:ident, $parts:ident| $body:block) => {
        pub struct $name;

        impl $crate::RequestDecorator for $name {
            type Output = $output;

            fn extract(
                $ctx: &$crate::RequestContext,
                $parts: &axum::http::request::Parts,
            ) -> Result<Self::Output, $crate::HttpException> {
                $body
            }
        }
    };
}
#[cfg(feature = "mongo")]
pub use nestforge_mongo::{InMemoryMongoRepo, MongoConfig};
#[cfg(feature = "openapi")]
pub use nestforge_openapi::{docs_router, OpenApiDoc, OpenApiRoute};
#[cfg(feature = "graphql")]
pub use nestforge_graphql::{
    async_graphql, graphql_auth_identity, graphql_container, graphql_request_id, graphql_router,
    graphql_router_with_config, resolve_graphql, GraphQlConfig, GraphQlSchema,
};
#[cfg(feature = "grpc")]
pub use nestforge_grpc::{prost, tonic, GrpcContext, GrpcServerConfig, NestForgeGrpcFactory};
#[cfg(feature = "schedule")]
pub use nestforge_schedule::{
    shutdown_schedules, start_schedules, ScheduleRegistry, ScheduleRegistryBuilder,
    ScheduledJob, ScheduledJobKind,
};
#[cfg(feature = "websockets")]
pub use nestforge_websockets::{
    websocket_gateway_router, websocket_gateway_router_with_config, websocket_router,
    websocket_router_with_config, CloseFrame, Message, Utf8Bytes, WebSocket, WebSocketConfig,
    WebSocketContext, WebSocketGateway,
};
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
) -> anyhow::Result<axum::Router<Container>> {
    let doc = openapi_doc_for_module::<M>(title, version)?;
    Ok(docs_router(doc))
}

#[cfg(feature = "openapi")]
pub trait NestForgeFactoryOpenApiExt<M: ModuleDefinition> {
    fn with_openapi_docs(
        self,
        title: impl Into<String>,
        version: impl Into<String>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized;
}

#[cfg(feature = "openapi")]
impl<M: ModuleDefinition> NestForgeFactoryOpenApiExt<M> for NestForgeFactory<M> {
    fn with_openapi_docs(
        self,
        title: impl Into<String>,
        version: impl Into<String>,
    ) -> anyhow::Result<Self> {
        let router = openapi_docs_router_for_module::<M>(title, version)?;
        Ok(self.merge_router(router))
    }
}

#[cfg(feature = "graphql")]
pub trait NestForgeFactoryGraphQlExt<M: ModuleDefinition> {
    fn with_graphql<Query, Mutation, Subscription>(
        self,
        schema: GraphQlSchema<Query, Mutation, Subscription>,
    ) -> Self
    where
        Query: async_graphql::ObjectType + Send + Sync + 'static,
        Mutation: async_graphql::ObjectType + Send + Sync + 'static,
        Subscription: async_graphql::SubscriptionType + Send + Sync + 'static,
        Self: Sized;

    fn with_graphql_config<Query, Mutation, Subscription>(
        self,
        schema: GraphQlSchema<Query, Mutation, Subscription>,
        config: GraphQlConfig,
    ) -> Self
    where
        Query: async_graphql::ObjectType + Send + Sync + 'static,
        Mutation: async_graphql::ObjectType + Send + Sync + 'static,
        Subscription: async_graphql::SubscriptionType + Send + Sync + 'static,
        Self: Sized;
}

#[cfg(feature = "graphql")]
impl<M: ModuleDefinition> NestForgeFactoryGraphQlExt<M> for NestForgeFactory<M> {
    fn with_graphql<Query, Mutation, Subscription>(
        self,
        schema: GraphQlSchema<Query, Mutation, Subscription>,
    ) -> Self
    where
        Query: async_graphql::ObjectType + Send + Sync + 'static,
        Mutation: async_graphql::ObjectType + Send + Sync + 'static,
        Subscription: async_graphql::SubscriptionType + Send + Sync + 'static,
    {
        self.merge_router(graphql_router(schema))
    }

    fn with_graphql_config<Query, Mutation, Subscription>(
        self,
        schema: GraphQlSchema<Query, Mutation, Subscription>,
        config: GraphQlConfig,
    ) -> Self
    where
        Query: async_graphql::ObjectType + Send + Sync + 'static,
        Mutation: async_graphql::ObjectType + Send + Sync + 'static,
        Subscription: async_graphql::SubscriptionType + Send + Sync + 'static,
    {
        self.merge_router(graphql_router_with_config(schema, config))
    }
}

#[cfg(feature = "websockets")]
pub trait NestForgeFactoryWebSocketExt<M: ModuleDefinition> {
    fn with_websocket_gateway<G>(self, gateway: G) -> Self
    where
        G: WebSocketGateway,
        Self: Sized;

    fn with_websocket_gateway_config<G>(self, gateway: G, config: WebSocketConfig) -> Self
    where
        G: WebSocketGateway,
        Self: Sized;
}

#[cfg(feature = "websockets")]
impl<M: ModuleDefinition> NestForgeFactoryWebSocketExt<M> for NestForgeFactory<M> {
    fn with_websocket_gateway<G>(self, gateway: G) -> Self
    where
        G: WebSocketGateway,
    {
        self.merge_router(websocket_gateway_router(gateway))
    }

    fn with_websocket_gateway_config<G>(self, gateway: G, config: WebSocketConfig) -> Self
    where
        G: WebSocketGateway,
    {
        self.merge_router(websocket_gateway_router_with_config(gateway, config))
    }
}
