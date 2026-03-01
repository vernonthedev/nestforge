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
pub mod container;
pub mod documentation;
pub mod error;
pub mod http_ext;
pub mod inject;
pub mod logging;
pub mod module;
pub mod pipeline;
pub mod provider;
pub mod request;
pub mod resource_service;
pub mod route_builder;
pub mod store;
pub mod validation;

pub use auth::{AuthIdentity, AuthUser, BearerToken, OptionalAuthUser};
pub use container::{Container, ContainerError};
pub use documentation::{DocumentedController, RouteDocumentation, RouteResponseDocumentation};
pub use error::HttpException;
pub use http_ext::{OptionHttpExt, ResultHttpExt};
pub use inject::Inject;
pub use logging::{framework_log, framework_log_event};
pub use module::{
    collect_module_route_docs, initialize_module_graph, initialize_module_runtime,
    ControllerBasePath, ControllerDefinition, DynamicModuleBuilder, InitializedModule,
    LifecycleHook, ModuleDefinition, ModuleRef,
};
pub use pipeline::{
    apply_exception_filters, execute_pipeline, ExceptionFilter, Guard, Interceptor, NextFn,
    NextFuture, RequestContext, RequireAuthenticationGuard, RoleRequirementsGuard, run_guards,
};
pub use provider::{register_provider, Provider, RegisterProvider};
pub use request::ValidatedBody;
pub use request::{Body, Cookies, Headers, Param, Pipe, PipedBody, PipedParam, PipedQuery, Query, RequestId};
pub use resource_service::{ResourceError, ResourceService};
pub use route_builder::RouteBuilder;
pub use store::{Identifiable, InMemoryStore};
pub use validation::{Validate, ValidationErrors, ValidationIssue};

pub type ApiResult<T> = Result<axum::Json<T>, HttpException>;
pub type List<T> = Vec<T>;
