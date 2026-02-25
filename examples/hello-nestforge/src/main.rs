use axum::{
    extract::State,
    routing::get,
    Router,
};
use nestforge::{Container, ControllerDefinition, ModuleDefinition, NestForgeFactory};

/*
AppModule = root module (manual version for now)

It registers providers into DI and exposes controllers.
*/
struct AppModule;

impl ModuleDefinition for AppModule {
    fn register(container: &Container) -> anyhow::Result<()> {
        container.register(AppConfig {
            app_name: "NestForge".to_string(),
        })?;

        Ok(())
    }

    fn controllers() -> Vec<Router<Container>> {
        vec![
            AppController::router(),
            HealthController::router(),
        ]
    }
}

/*
A tiny config object stored in the DI container.
Controllers can read this from state.
*/
#[derive(Clone)]
struct AppConfig {
    app_name: String,
}

/*
AppController = handles "/" route
Nest-style vibe: route logic grouped in a controller struct.
*/
struct AppController;

impl ControllerDefinition for AppController {
    fn router() -> Router<Container> {
        Router::new().route("/", get(Self::root))
    }
}

impl AppController {
    /*
    We pull the Container from axum state, then resolve AppConfig from DI.
    */
    async fn root(State(container): State<Container>) -> String {
        match container.resolve::<AppConfig>() {
            Ok(cfg) => format!("Welcome to {}", cfg.app_name),
            Err(_) => "Welcome to NestForge 🦀".to_string(),
        }
    }
}

/*
HealthController = handles "/health"
*/
struct HealthController;

impl ControllerDefinition for HealthController {
    fn router() -> Router<Container> {
        Router::new().route("/health", get(Self::health))
    }
}

impl HealthController {
    async fn health() -> &'static str {
        "OK"
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    /*
    Quick DI sanity check (still useful while building)
    */
    let container = Container::new();
    container.register(AppConfig {
        app_name: "NestForge".into(),
    })?;
    let cfg = container.resolve::<AppConfig>()?;
    println!("Booting {}", cfg.app_name);

    /*
    Framework boot using AppModule
    */
    NestForgeFactory::<AppModule>::create()?.listen(3000).await
}