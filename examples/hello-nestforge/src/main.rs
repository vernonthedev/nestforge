use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use nestforge::{Container, ControllerDefinition, ModuleDefinition, NestForgeFactory};
use serde::Serialize;

/*
AppModule = root module (manual version for now)

This is where we:
1) register providers/services in DI
2) expose controllers
*/
struct AppModule;

impl ModuleDefinition for AppModule {
    fn register(container: &Container) -> anyhow::Result<()> {
        /*
        Register app config
        */
        container.register(AppConfig {
            app_name: "NestForge".to_string(),
        })?;

        /*
        Register UsersService (our first real service/provider)

        This is very Nest-like:
        provider gets registered in the module,
        controller will resolve and use it.
        */
        container.register(UsersService::new())?;

        Ok(())
    }

    fn controllers() -> Vec<Router<Container>> {
        vec![
            AppController::router(),
            HealthController::router(),
            UsersController::router(),
        ]
    }
}

/*
A tiny config object stored in DI
*/
#[derive(Clone)]
struct AppConfig {
    app_name: String,
}

/*
AppController = handles "/"
*/
struct AppController;

impl ControllerDefinition for AppController {
    fn router() -> Router<Container> {
        Router::new().route("/", get(Self::root))
    }
}

impl AppController {
    async fn root(State(container): State<Container>) -> String {
        match container.resolve::<AppConfig>() {
            Ok(cfg) => format!("Welcome to {} 🔥", cfg.app_name),
            Err(_) => "Welcome to NestForge 🔥".to_string(),
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

/*
UserDto = response shape returned to the client

This is the "DTO" vibe (manual version for now).
Serialize lets axum turn it into JSON.
*/
#[derive(Debug, Clone, Serialize)]
struct UserDto {
    id: u64,
    name: String,
    email: String,
}

/*
UsersService = business logic layer (provider)

NestJS vibe:
- controller should not own data/business logic
- service handles that part
*/
#[derive(Clone)]
struct UsersService;

impl UsersService {
    fn new() -> Self {
        Self
    }

    /*
    Pretend this is fetching from a DB.
    For now we return mock data so the flow is clear.
    */
    fn find_all(&self) -> Vec<UserDto> {
        vec![
            UserDto {
                id: 1,
                name: "Vernon".to_string(),
                email: "vernonthedev@gmail.com".to_string(),
            },
            UserDto {
                id: 2,
                name: "John".to_string(),
                email: "johndoe@mail.com".to_string(),
            },
        ]
    }
}

/*
UsersController = handles "/users"

Controller pulls service from DI container and returns JSON.
*/
struct UsersController;

impl ControllerDefinition for UsersController {
    fn router() -> Router<Container> {
        Router::new().route("/users", get(Self::list_users))
    }
}

impl UsersController {
    /*
    Flow:
    - get Container from app state
    - resolve UsersService from DI
    - call service method
    - return JSON response
    */
    async fn list_users(State(container): State<Container>) -> Json<Vec<UserDto>> {
        let users = match container.resolve::<UsersService>() {
            Ok(service) => service.find_all(),
            Err(_) => {
                /*
                If DI fails for some reason, return empty list for now.
                Later we’ll return proper HTTP errors.
                */
                Vec::new()
            }
        };

        Json(users)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    /*
    Quick DI sanity check while we build the framework
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