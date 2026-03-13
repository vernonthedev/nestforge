use std::{
    future::Future,
    net::SocketAddr,
    pin::Pin,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use axum::{
    body::Body,
    http::{header::HeaderName, HeaderValue},
    middleware::from_fn,
    response::{IntoResponse, Response},
    Router,
};
use nestforge_core::{
    apply_exception_filters, execute_pipeline, framework_log_event, initialize_module_runtime,
    AuthIdentity, Container, ExceptionFilter, Guard, HttpException, InitializedModule, Interceptor,
    ModuleDefinition, NextFn, RequestContext, RequestId,
};

use crate::middleware::{
    run_middleware_chain, MiddlewareBinding, MiddlewareConsumer, NestMiddleware,
};

/// The main entry point for creating a NestForge application.
///
/// It handles the bootstrap process:
/// 1. Creating the DI Container.
/// 2. Initializing the Module Graph (resolving imports and providers).
/// 3. Merging all Controller routers into a single Axum app.
/// 4. Attaching global middleware, guards, interceptors, and exception filters.
///
/// # Example
/// ```rust,no_run
/// use nestforge_http::NestForgeFactory;
///
/// #[tokio::main]
/// async fn main() {
///     let app = NestForgeFactory::<AppModule>::create()
///         .expect("failed to start")
///         .listen(3000)
///         .await;
/// }
/// ```
pub struct NestForgeFactory<M: ModuleDefinition> {
    _marker: std::marker::PhantomData<M>,
    container: Container,
    runtime: Arc<InitializedModule>,
    controllers: Vec<Router<Container>>,
    extra_routers: Vec<Router<Container>>,
    global_prefix: Option<String>,
    version: Option<String>,
    auth_resolver: Option<Arc<AuthResolver>>,
    global_guards: Vec<Arc<dyn Guard>>,
    global_interceptors: Vec<Arc<dyn Interceptor>>,
    global_exception_filters: Vec<Arc<dyn ExceptionFilter>>,
    middleware_bindings: Vec<MiddlewareBinding>,
}

type AuthFuture = Pin<Box<dyn Future<Output = Result<Option<AuthIdentity>, HttpException>> + Send>>;
type AuthResolver = dyn Fn(Option<String>, Container) -> AuthFuture + Send + Sync;

impl<M: ModuleDefinition> NestForgeFactory<M> {
    /// Creates a new application instance from the root module.
    ///
    /// This triggers the DI container initialization and module lifecycle hooks (e.g., `on_module_init`).
    pub fn create() -> Result<Self> {
        let container = Container::new();
        let runtime = Arc::new(initialize_module_runtime::<M>(&container)?);
        runtime.run_module_init(&container)?;
        runtime.run_application_bootstrap(&container)?;
        let controllers = runtime.controllers.clone();

        Ok(Self {
            _marker: std::marker::PhantomData,
            container,
            runtime,
            controllers,
            extra_routers: Vec::new(),
            global_prefix: None,
            version: None,
            auth_resolver: None,
            global_guards: Vec::new(),
            global_interceptors: Vec::new(),
            global_exception_filters: Vec::new(),
            middleware_bindings: Vec::new(),
        })
    }

    /// Sets a global prefix for all routes (e.g., "api").
    pub fn with_global_prefix(mut self, prefix: impl Into<String>) -> Self {
        let prefix = prefix.into().trim().trim_matches('/').to_string();
        if !prefix.is_empty() {
            framework_log_event("global_prefix_configured", &[("prefix", prefix.clone())]);
            self.global_prefix = Some(prefix);
        }
        self
    }

    /// Sets a global API version for all routes (e.g., "v1").
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        let version = version.into().trim().trim_matches('/').to_string();
        if !version.is_empty() {
            framework_log_event("api_version_configured", &[("version", version.clone())]);
            self.version = Some(version);
        }
        self
    }

    /// Registers a global guard.
    ///
    /// Global guards run for *every* route in the application.
    pub fn use_guard<G>(mut self) -> Self
    where
        G: Guard + Default,
    {
        framework_log_event(
            "global_guard_register",
            &[("guard", std::any::type_name::<G>().to_string())],
        );
        self.global_guards.push(Arc::new(G::default()));
        self
    }

    /// Registers a global interceptor.
    ///
    /// Global interceptors wrap *every* route handler.
    pub fn use_interceptor<I>(mut self) -> Self
    where
        I: Interceptor + Default,
    {
        framework_log_event(
            "global_interceptor_register",
            &[("interceptor", std::any::type_name::<I>().to_string())],
        );
        self.global_interceptors.push(Arc::new(I::default()));
        self
    }

    /// Registers a global exception filter.
    ///
    /// Catches unhandled exceptions from *any* route.
    pub fn use_exception_filter<F>(mut self) -> Self
    where
        F: ExceptionFilter + Default,
    {
        framework_log_event(
            "global_exception_filter_register",
            &[("filter", std::any::type_name::<F>().to_string())],
        );
        self.global_exception_filters.push(Arc::new(F::default()));
        self
    }

    /// Applies middleware to the application.
    ///
    /// Use the builder to select which routes the middleware applies to.
    pub fn use_middleware<T>(mut self) -> Self
    where
        T: NestMiddleware + Default,
    {
        let mut consumer = MiddlewareConsumer::new();
        consumer.apply::<T>().for_all_routes();
        self.middleware_bindings.extend(consumer.into_bindings());
        self
    }

    /// Advanced middleware configuration using a consumer builder.
    pub fn configure_middleware<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(&mut MiddlewareConsumer),
    {
        let mut consumer = MiddlewareConsumer::new();
        configure(&mut consumer);
        self.middleware_bindings.extend(consumer.into_bindings());
        self
    }

    /// Sets the authentication resolver.
    ///
    /// This function is called for every request to resolve the `AuthIdentity`
    /// from the bearer token.
    pub fn with_auth_resolver<F, Fut>(mut self, resolver: F) -> Self
    where
        F: Fn(Option<String>, Container) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Option<AuthIdentity>, HttpException>> + Send + 'static,
    {
        self.auth_resolver = Some(Arc::new(move |token, container| {
            Box::pin(resolver(token, container))
        }));
        self
    }

    /// Merges an external Axum router into the application.
    ///
    /// Useful for integrating other libraries or raw Axum handlers.
    pub fn merge_router(mut self, router: Router<Container>) -> Self {
        self.extra_routers.push(router);
        self
    }

    /// Returns a reference to the underlying DI Container.
    pub fn container(&self) -> &Container {
        &self.container
    }

    /// Consumes the factory and returns the fully configured Axum Router.
    ///
    /// Use this if you want to run the app with your own server (e.g. Lambda, Shuttle).
    pub fn into_router(self) -> Router {
        /*
        Build a router that EXPECTS Container state.
        We don't attach the actual state yet.
        */
        let mut app: Router<Container> = Router::new();

        /*
        Mount all controller routers (they are also Router<Container>)
        */
        for controller_router in self.controllers {
            app = app.merge(controller_router);
        }
        for extra_router in self.extra_routers {
            app = app.merge(extra_router);
        }

        if let Some(version) = &self.version {
            app = Router::new().nest(&format!("/{}", version), app);
        }

        if let Some(prefix) = &self.global_prefix {
            app = Router::new().nest(&format!("/{}", prefix), app);
        }

        let global_guards = Arc::new(self.global_guards);
        let global_interceptors = Arc::new(self.global_interceptors);
        let global_exception_filters = Arc::new(self.global_exception_filters);
        let middleware_bindings = Arc::new(self.middleware_bindings);
        let auth_resolver = self.auth_resolver.clone();
        let request_container = self.container.clone();

        let route_exception_filters = Arc::clone(&global_exception_filters);
        let app = app.route_layer(from_fn(move |req, next| {
            let guards = Arc::clone(&global_guards);
            let interceptors = Arc::clone(&global_interceptors);
            let filters = Arc::clone(&route_exception_filters);
            async move { execute_pipeline(req, next, guards, interceptors, filters).await }
        }));

        let app = app.layer(from_fn(
            move |req: axum::extract::Request, next: axum::middleware::Next| {
                let middlewares = Arc::clone(&middleware_bindings);
                async move {
                    if middlewares.is_empty() {
                        return next.run(req).await;
                    }

                    let terminal = next_to_fn(next);
                    run_middleware_chain(middlewares, 0, req, terminal).await
                }
            },
        ));

        Router::new()
            .merge(app)
            .layer(from_fn(move |req, next| {
                let auth_resolver = auth_resolver.clone();
                let request_container = request_container.clone();
                let exception_filters = Arc::clone(&global_exception_filters);
                async move {
                    request_context_middleware(
                        req,
                        next,
                        request_container,
                        auth_resolver,
                        exception_filters,
                    )
                    .await
                }
            }))
            .with_state(self.container)
    }

    /// Starts the HTTP server on the specified port.
    ///
    /// This will block the current thread (it should be awaited).
    /// Upon shutdown (Ctrl+C), it runs the `on_module_destroy` and `on_application_shutdown` hooks.
    pub async fn listen(self, port: u16) -> Result<()> {
        let runtime = Arc::clone(&self.runtime);
        let container = self.container.clone();
        let app = self.into_router();

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr).await?;

        framework_log_event("server_listening", &[("addr", addr.to_string())]);

        axum::serve(listener, app).await?;
        runtime.run_module_destroy(&container)?;
        runtime.run_application_shutdown(&container)?;
        Ok(())
    }
}

static NEXT_REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);
const REQUEST_ID_HEADER: &str = "x-request-id";

fn next_to_fn(next: axum::middleware::Next) -> NextFn {
    let next = Arc::new(Mutex::new(Some(next)));

    Arc::new(move |req: axum::extract::Request<Body>| {
        let next = Arc::clone(&next);
        Box::pin(async move {
            let next = {
                let mut guard = match next.lock() {
                    Ok(guard) => guard,
                    Err(_) => {
                        return HttpException::internal_server_error("Middleware lock poisoned")
                            .into_response();
                    }
                };
                guard.take()
            };

            match next {
                Some(next) => next.run(req).await,
                None => {
                    HttpException::internal_server_error("Middleware next called multiple times")
                        .into_response()
                }
            }
        })
    })
}

async fn request_context_middleware(
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
    container: Container,
    auth_resolver: Option<Arc<AuthResolver>>,
    exception_filters: Arc<Vec<Arc<dyn ExceptionFilter>>>,
) -> Response {
    let scoped_container = container.scoped();
    let request_id = RequestId::new(generate_request_id());
    let request_id_value = request_id.value().to_string();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let started = Instant::now();
    let bearer_token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    req.extensions_mut().insert(scoped_container.clone());
    req.extensions_mut().insert(request_id.clone());
    let _ = scoped_container.override_value(request_id.clone());
    framework_log_event(
        "request_start",
        &[
            ("request_id", request_id_value.clone()),
            ("method", method.clone()),
            ("path", path.clone()),
        ],
    );

    if let Some(resolver) = auth_resolver {
        match resolver(bearer_token, container).await {
            Ok(Some(identity)) => {
                framework_log_event(
                    "auth_identity_resolved",
                    &[
                        ("request_id", request_id_value.clone()),
                        ("subject", identity.subject.clone()),
                    ],
                );
                let _ = scoped_container.override_value(identity.clone());
                req.extensions_mut().insert(Arc::new(identity));
            }
            Ok(None) => {}
            Err(err) => {
                let ctx = RequestContext::from_request(&req);
                let _ = scoped_container.override_value(ctx.clone());
                let mut response = apply_exception_filters(
                    err.with_request_id(request_id_value.clone()),
                    &ctx,
                    exception_filters.as_slice(),
                )
                .into_response();
                attach_request_id_header(&mut response, &request_id_value);
                framework_log_event(
                    "request_complete",
                    &[
                        ("request_id", request_id_value),
                        ("method", method),
                        ("path", path),
                        ("status", response.status().as_u16().to_string()),
                        ("duration_ms", started.elapsed().as_millis().to_string()),
                    ],
                );
                return response;
            }
        }
    }

    let ctx = RequestContext::from_request(&req);
    let _ = scoped_container.override_value(ctx);

    let mut response = next.run(req).await;
    attach_request_id_header(&mut response, &request_id_value);

    framework_log_event(
        "request_complete",
        &[
            ("request_id", request_id_value),
            ("method", method),
            ("path", path),
            ("status", response.status().as_u16().to_string()),
            ("duration_ms", started.elapsed().as_millis().to_string()),
        ],
    );

    response
}

fn generate_request_id() -> String {
    let sequence = NEXT_REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("req-{millis}-{sequence}")
}

fn attach_request_id_header(response: &mut Response, request_id: &str) {
    if let Ok(value) = HeaderValue::from_str(request_id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static(REQUEST_ID_HEADER), value);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::Result;
    use axum::Json;
    use nestforge_core::{
        register_provider, ApiResult, AuthUser, Container, ControllerBasePath,
        ControllerDefinition, ExceptionFilter, HttpException, Inject, ModuleDefinition, Provider,
        RequestContext as FrameworkRequestContext, RouteBuilder,
    };
    use tower::ServiceExt;

    use super::*;

    struct HealthController;
    #[derive(Default)]
    struct RewriteBadRequestFilter;
    struct RequestScopedService {
        path: String,
    }

    impl ControllerBasePath for HealthController {
        fn base_path() -> &'static str {
            "/health"
        }
    }

    impl HealthController {
        async fn ok(request_id: RequestId) -> ApiResult<String> {
            Ok(Json(request_id.value().to_string()))
        }

        async fn fail(request_id: RequestId) -> ApiResult<String> {
            Err(HttpException::bad_request("broken request")
                .with_request_id(request_id.value().to_string()))
        }

        async fn fail_locally(request_id: RequestId) -> ApiResult<String> {
            Err(HttpException::bad_request("local broken request")
                .with_request_id(request_id.value().to_string()))
        }

        async fn me(user: AuthUser) -> ApiResult<String> {
            Ok(Json(user.subject.clone()))
        }

        async fn scoped(service: Inject<RequestScopedService>) -> ApiResult<String> {
            Ok(Json(service.path.clone()))
        }
    }

    impl ControllerDefinition for HealthController {
        fn router() -> Router<Container> {
            RouteBuilder::<Self>::new()
                .get("/", Self::ok)
                .get("/fail", Self::fail)
                .get_with_pipeline(
                    "/fail-local",
                    Self::fail_locally,
                    Vec::new(),
                    Vec::new(),
                    vec![Arc::new(RewriteBadRequestFilter) as Arc<dyn ExceptionFilter>],
                    None,
                )
                .get("/me", Self::me)
                .get("/scoped", Self::scoped)
                .build()
        }
    }

    impl ExceptionFilter for RewriteBadRequestFilter {
        fn catch(&self, exception: HttpException, _ctx: &RequestContext) -> HttpException {
            if exception.status == axum::http::StatusCode::BAD_REQUEST {
                HttpException::bad_request("filtered bad request")
                    .with_optional_request_id(exception.request_id)
            } else {
                exception
            }
        }
    }

    struct TestModule;

    impl ModuleDefinition for TestModule {
        fn register(container: &Container) -> Result<()> {
            register_provider(
                container,
                Provider::request_factory(|container| {
                    let ctx = container.resolve::<FrameworkRequestContext>()?;
                    Ok(RequestScopedService {
                        path: ctx.uri.path().to_string(),
                    })
                }),
            )?;
            Ok(())
        }

        fn controllers() -> Vec<Router<Container>> {
            vec![HealthController::router()]
        }
    }

    #[tokio::test]
    async fn request_middleware_sets_request_id_header_and_extension() {
        let app = NestForgeFactory::<TestModule>::create()
            .expect("factory should build")
            .into_router();

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/health/")
                    .body(axum::body::Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert!(response.headers().contains_key(REQUEST_ID_HEADER));
    }

    #[tokio::test]
    async fn error_responses_keep_request_id_header() {
        let app = NestForgeFactory::<TestModule>::create()
            .expect("factory should build")
            .use_exception_filter::<RewriteBadRequestFilter>()
            .into_router();

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/health/fail")
                    .body(axum::body::Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
        assert!(response.headers().contains_key(REQUEST_ID_HEADER));
    }

    #[tokio::test]
    async fn route_specific_exception_filters_rewrite_route_failures() {
        let app = NestForgeFactory::<TestModule>::create()
            .expect("factory should build")
            .into_router();

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/health/fail-local")
                    .body(axum::body::Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
        assert!(response.headers().contains_key(REQUEST_ID_HEADER));
    }

    #[tokio::test]
    async fn auth_resolver_inserts_identity_for_auth_user_extractor() {
        let app = NestForgeFactory::<TestModule>::create()
            .expect("factory should build")
            .with_auth_resolver(|token, _container| async move {
                Ok(token.map(|_| AuthIdentity::new("demo-user").with_roles(["admin"])))
            })
            .into_router();

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/health/me")
                    .header(axum::http::header::AUTHORIZATION, "Bearer demo-token")
                    .body(axum::body::Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn request_scoped_provider_resolves_from_per_request_container() {
        let app = NestForgeFactory::<TestModule>::create()
            .expect("factory should build")
            .into_router();

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/health/scoped")
                    .body(axum::body::Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }
}
