use axum::Router;
use nestforge::{Container, ControllerDefinition, ModuleDefinition};

use crate::{
    controllers::{AppController, HealthController, UsersController},
    services::{AppConfig, UsersService},
};

pub struct AppModule;

impl ModuleDefinition for AppModule {
    fn register(container: &Container) -> anyhow::Result<()> {
        container.register(AppConfig {
            app_name: "NestForge".to_string(),
        })?;

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