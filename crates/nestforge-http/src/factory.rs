use std::{
    net::SocketAddr,
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
    response::Response,
    Router,
};
use nestforge_core::{
    execute_pipeline, framework_log_event, initialize_module_graph, Container, Guard, Interceptor,
    ModuleDefinition, RequestId,
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
    global_guards: Vec<Arc<dyn Guard>>,
    global_interceptors: Vec<Arc<dyn Interceptor>>,
}

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

        let app = app.route_layer(from_fn(move |req, next| {
            let guards = Arc::clone(&global_guards);
            let interceptors = Arc::clone(&global_interceptors);
            async move { execute_pipeline(req, next, guards, interceptors).await }
        }));

        Router::new()
            .merge(app)
            .layer(from_fn(request_context_middleware))
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
) -> Response {
    let request_id = RequestId::new(generate_request_id());
    let request_id_value = request_id.value().to_string();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let started = Instant::now();

    req.extensions_mut().insert(request_id);
    framework_log_event(
        "request_start",
        &[
            ("request_id", request_id_value.clone()),
            ("method", method.clone()),
            ("path", path.clone()),
        ],
    );

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
        ApiResult, Container, ControllerBasePath, ControllerDefinition, HttpException,
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
    }

    impl ControllerDefinition for HealthController {
        fn router() -> Router<Container> {
            RouteBuilder::<Self>::new()
                .get("/", Self::ok)
                .get("/fail", Self::fail)
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
}
