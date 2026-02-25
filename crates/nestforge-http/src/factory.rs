use std::net::SocketAddr;

use anyhow::Result;
use axum::Router;
use nestforge_core::{Container, ModuleDefinition};
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
}

impl<M: ModuleDefinition> NestForgeFactory<M> {
    pub fn create() -> Result<Self> {
        let container = Container::new();

        /* Let the module register providers/services into DI */
        M::register(&container)?;

        Ok(Self {
            _marker: std::marker::PhantomData,
            container,
        })
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
        for controller_router in M::controllers() {
            app = app.merge(controller_router);
        }

        /*
        Now attach the real Container state.
        After this, the router becomes ready to run.
        */
        let app = app
            .with_state(self.container.clone())
            .layer(TraceLayer::new_for_http());

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr).await?;

        println!("🦀 NestForge running on http://{}", addr);

        axum::serve(listener, app).await?;
        Ok(())
    }
}