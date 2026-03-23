/**
* This is the entry file for nestforge-core.
* - DI container
* - module traits
* - controller traits
* - framework error types   
* - DI helpers
* - In memory store
*/
pub mod auth;
pub mod config;
pub mod container;
pub mod documentation;
pub mod error;
pub mod http_ext;
pub mod inject;
pub mod injectable;
pub mod logging;
pub mod module;
pub mod pipeline;
pub mod provider;
pub mod request;
pub mod resource_service;
pub mod response;
pub mod route_builder;
pub mod store;
pub mod validation;

pub use auth::{AuthIdentity, AuthUser, BearerToken, OptionalAuthUser};
pub use config::{register_config, Configurable};
pub use container::{Container, ContainerError};
pub use documentation::{
    openapi_array_schema_for, openapi_nullable_schema_for, openapi_schema_components_for,
    openapi_schema_for, DocumentedController, OpenApiSchema, OpenApiSchemaComponent,
    RouteDocumentation, RouteResponseDocumentation,
};
pub use error::HttpException;
pub use http_ext::{OptionHttpExt, ResultHttpExt};
pub use inject::Inject;
pub use injectable::{register_injectable, Injectable, IntoInjectableResult};
pub use logging::{framework_log, framework_log_event};
pub use module::{
    collect_module_graph, collect_module_route_docs, initialize_module_graph,
    initialize_module_runtime, ControllerBasePath, ControllerDefinition, DynamicModuleBuilder,
    InitializedModule, LifecycleHook, ModuleDefinition, ModuleGraphEntry, ModuleGraphReport,
    ModuleRef,
};
pub use pipeline::{
    apply_exception_filters, execute_pipeline, run_guards, ExceptionFilter, Guard, Interceptor,
    NextFn, NextFuture, RequestContext, RequireAuthenticationGuard, RoleRequirementsGuard,
};
pub use provider::{register_provider, Provider, RegisterProvider};
pub use request::ValidatedBody;
pub use request::{
    Body, Cookies, Decorated, Headers, Param, Pipe, PipedBody, PipedParam, PipedQuery, Query,
    RequestDecorator, RequestId,
};
pub use resource_service::{ResourceError, ResourceService};
pub use response::{
    ApiEnvelopeResult, ApiSerializedResult, ResponseEnvelope, ResponseSerializer, Serialized,
};
pub use route_builder::RouteBuilder;
pub use store::{Identifiable, InMemoryStore};
pub use validation::{Validate, ValidationErrors, ValidationIssue};
pub use nestforge_config::{
    ConfigError, ConfigModule, ConfigOptions, EnvStore, FromEnv, EnvValidationIssue,
};

pub type ApiResult<T> = Result<axum::Json<T>, HttpException>;
pub type List<T> = Vec<T>;
