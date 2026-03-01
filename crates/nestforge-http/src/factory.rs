use std::{
    future::Future,
    net::SocketAddr,
    pin::Pin,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use axum::{
    http::{header::HeaderName, HeaderValue},
    middleware::from_fn,
    response::{IntoResponse, Response},
    Router,
};
use nestforge_core::{
    execute_pipeline, framework_log_event, initialize_module_graph, AuthIdentity, Container,
    Guard, HttpException, Interceptor, ModuleDefinition, RequestId,
};

/*
NestForgeFactory = app bootstrapper.

This is the NestFactory.create(AppModule) vibe.

Now it:
- builds DI container
- asks the module to register providers
- asks the module for controllers
- merges controller routers into one app router
*/
pub struct NestForgeFactory<M: ModuleDefinition> {
    _marker: std::marker::PhantomData<M>,
    container: Container,
    controllers: Vec<Router<Container>>,
    global_prefix: Option<String>,
    version: Option<String>,
    auth_resolver: Option<Arc<AuthResolver>>,
    global_guards: Vec<Arc<dyn Guard>>,
    global_interceptors: Vec<Arc<dyn Interceptor>>,
}

type AuthFuture = Pin<Box<dyn Future<Output = Result<Option<AuthIdentity>, HttpException>> + Send>>;
type AuthResolver = dyn Fn(Option<String>, Container) -> AuthFuture + Send + Sync;

impl<M: ModuleDefinition> NestForgeFactory<M> {
    pub fn create() -> Result<Self> {
        let container = Container::new();
        let controllers = initialize_module_graph::<M>(&container)?;

        Ok(Self {
            _marker: std::marker::PhantomData,
            container,
            controllers,
            global_prefix: None,
            version: None,
            auth_resolver: None,
            global_guards: Vec::new(),
            global_interceptors: Vec::new(),
        })
    }

    pub fn with_global_prefix(mut self, prefix: impl Into<String>) -> Self {
        let prefix = prefix.into().trim().trim_matches('/').to_string();
        if !prefix.is_empty() {
            framework_log_event(
                "global_prefix_configured",
                &[("prefix", prefix.clone())],
            );
            self.global_prefix = Some(prefix);
        }
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        let version = version.into().trim().trim_matches('/').to_string();
        if !version.is_empty() {
            framework_log_event(
                "api_version_configured",
                &[("version", version.clone())],
            );
            self.version = Some(version);
        }
        self
    }

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

        if let Some(version) = &self.version {
            app = Router::new().nest(&format!("/{}", version), app);
        }

        if let Some(prefix) = &self.global_prefix {
            app = Router::new().nest(&format!("/{}", prefix), app);
        }

        let global_guards = Arc::new(self.global_guards);
        let global_interceptors = Arc::new(self.global_interceptors);
        let auth_resolver = self.auth_resolver.clone();
        let request_container = self.container.clone();

        let app = app.route_layer(from_fn(move |req, next| {
            let guards = Arc::clone(&global_guards);
            let interceptors = Arc::clone(&global_interceptors);
            async move { execute_pipeline(req, next, guards, interceptors).await }
        }));

        Router::new()
            .merge(app)
            .layer(from_fn(move |req, next| {
                let auth_resolver = auth_resolver.clone();
                let request_container = request_container.clone();
                async move {
                    request_context_middleware(req, next, request_container, auth_resolver).await
                }
            }))
            .with_state(self.container)
    }

    pub async fn listen(self, port: u16) -> Result<()> {
        let app = self.into_router();

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr).await?;

        framework_log_event("server_listening", &[("addr", addr.to_string())]);

        axum::serve(listener, app).await?;
        Ok(())
    }
}

static NEXT_REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);
const REQUEST_ID_HEADER: &str = "x-request-id";

async fn request_context_middleware(
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
    container: Container,
    auth_resolver: Option<Arc<AuthResolver>>,
) -> Response {
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

    req.extensions_mut().insert(request_id);
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
                req.extensions_mut().insert(Arc::new(identity));
            }
            Ok(None) => {}
            Err(err) => {
                let mut response = err
                    .with_request_id(request_id_value.clone())
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
        response.headers_mut().insert(
            HeaderName::from_static(REQUEST_ID_HEADER),
            value,
        );
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use axum::Json;
    use nestforge_core::{
        ApiResult, AuthUser, Container, ControllerBasePath, ControllerDefinition, HttpException,
        ModuleDefinition, RouteBuilder,
    };
    use tower::ServiceExt;

    use super::*;

    struct HealthController;

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

        async fn me(user: AuthUser) -> ApiResult<String> {
            Ok(Json(user.subject.clone()))
        }
    }

    impl ControllerDefinition for HealthController {
        fn router() -> Router<Container> {
            RouteBuilder::<Self>::new()
                .get("/", Self::ok)
                .get("/fail", Self::fail)
                .get("/me", Self::me)
                .build()
        }
    }

    struct TestModule;

    impl ModuleDefinition for TestModule {
        fn register(_container: &Container) -> Result<()> {
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
}
