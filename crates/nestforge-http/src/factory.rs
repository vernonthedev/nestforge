use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use axum::middleware::from_fn;
use nestforge_core::{
    execute_pipeline, framework_log, initialize_module_graph, Container, Guard, Interceptor,
    ModuleDefinition,
};
use tower_http::trace::TraceLayer;

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
            framework_log(format!("Using global route prefix '{}'.", prefix));
            self.global_prefix = Some(prefix);
        }
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        let version = version.into().trim().trim_matches('/').to_string();
        if !version.is_empty() {
            framework_log(format!("Using api version '{}'.", version));
            self.version = Some(version);
        }
        self
    }

    pub fn use_guard<G>(mut self) -> Self
    where
        G: Guard + Default,
    {
        framework_log(format!(
            "Registering guard {}.",
            std::any::type_name::<G>()
        ));
        self.global_guards.push(Arc::new(G::default()));
        self
    }

    pub fn use_interceptor<I>(mut self) -> Self
    where
        I: Interceptor + Default,
    {
        framework_log(format!(
            "Registering interceptor {}.",
            std::any::type_name::<I>()
        ));
        self.global_interceptors.push(Arc::new(I::default()));
        self
    }

    pub async fn listen(self, port: u16) -> Result<()> {
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

        let app = app
            .with_state(self.container.clone())
            .layer(TraceLayer::new_for_http());

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr).await?;

        framework_log(format!("Listening on {}.", addr));

        axum::serve(listener, app).await?;
        Ok(())
    }
}
