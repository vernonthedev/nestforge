use axum::Router;
use nestforge::{Container, ControllerDefinition, ModuleDefinition};

use crate::{
    controllers::{AppController, HealthController, UsersController},
    services::{AppConfig, UsersService},
};

/*
AppModule = root module for this example app.

This is where we:
- register providers/services into DI
- expose controllers (routers)
*/
pub struct AppModule;

impl ModuleDefinition for AppModule {
    fn register(container: &Container) -> anyhow::Result<()> {
        /*
        Register app-level config
        */
        container.register(AppConfig {
            app_name: "NestForge".to_string(),
        })?;

        /*
        Register service/provider
        */
        container.register(UsersService::new())?;

        Ok(())
    }

    fn controllers() -> Vec<Router<Container>> {
        /*
        These .router() methods come from the ControllerDefinition trait,
        so we imported the trait above to make them available here.
        */
        vec![
            AppController::router(),
            HealthController::router(),
            UsersController::router(),
        ]
    }
}