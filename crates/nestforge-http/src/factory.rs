use std::net::SocketAddr;

use anyhow::Result;
use axum::{routing::get, Router};
use nestforge_core::{Container, ModuleDefinition};
use tower_http::trace::TraceLayer;

/**
* NestForgeFactory = the app bootstrapper.
* 
* This is the equivalent vibe of:
* NestFactory.create(AppModule) in NestJS
* 
* Right now it:
* - creates a container.
* - asks the module to register providers.
* - starts an axum server.
*/
pub struct NestForgeFactory<M: ModuleDefinition> {
    /**
    * PhantomData tells Rust:
    * "This struct is logically tied to M"
    * even though we don’t store an actual M value inside.
    */
    _marker: std::marker::PhantomData<M>,

    /**
    * Shared app container (services/configs live here).
    */
    container: Container,
}

impl<M: ModuleDefinition> NestForgeFactory<M> {
    /**
    * create() prepares the app before the server starts.
    *
    * What happens here:
    * 1) create the DI container.
    * 2) let the AppModule register services/providers.
    * 3) return the factory ready to listen
    */
    pub fn create() -> Result<Self> {
        let container = Container::new();

        /* AppModule registers providers/services here. */
        M::register(&container)?;

        Ok(Self {
            _marker: std::marker::PhantomData,
            container,
        })
    }

    /**
    * listen(port) starts the HTTP server.
    *
    * - routes are hardcoded here just to prove everything works for now.
    * - later we’ll move routes into controller definitions.
    */
    pub async fn listen(self, port: u16) -> Result<()> {
        let app = Router::new()
            /* homepage route. */
            .route("/", get(root))
            /* health-check route. */
            .route("/health", get(health))
            /*
            TraceLayer logs requests in the console.
            */
            .layer(TraceLayer::new_for_http())
            /*
            Attach the container as app state.
            Later controllers/handlers can access it.
            */
            .with_state(self.container);

        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        /*
        Bind a TCP listener (actual network socket).
        */
        let listener = tokio::net::TcpListener::bind(addr).await?;

        println!("NestForge running on http://{}", addr);

        /*
        Start serving requests forever (until app stops).
        */
        axum::serve(listener, app).await?;
        Ok(())
    }
}

/**
* Simple route handler for "/"
*/
async fn root() -> &'static str {
    "Welcome to NestForge"
}

/**
* Simple route handler for "/health"
*/
async fn health() -> &'static str {
    "OK"
}