use nestforge::{Container, ModuleDefinition, NestForgeFactory};

/**
* AppModule = our first module (manual version for now).
*
* In NestJS terms, this is the root module.
* It registers app-level providers into the container.
*/
struct AppModule;

impl ModuleDefinition for AppModule {
    fn register(container: &Container) -> anyhow::Result<()> {
        /*
        * Register a basic config object into DI.
        * This is just a demo provider for now.
        */
        container.register(AppConfig {
            app_name: "NestForge".to_string(),
        })?;

        Ok(())
    }
}

/**
* A tiny config/service struct just to prove DI works.
*/
#[derive(Clone)]
struct AppConfig {
    app_name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    /*
    * Quick DI sanity check (separate from framework boot).
    *
    * This part is just us proving:
    * - register works
    * - resolve works
    */
    let container = Container::new();
    container.register(AppConfig {
        app_name: "NestForge".into(),
    })?;

    let cfg = container.resolve::<AppConfig>()?;
    println!("Booting {}", cfg.app_name);

    /*
    * Real framework boot:
    * - create app using AppModule
    * - start listening on port 3000
    */
    NestForgeFactory::<AppModule>::create()?.listen(3000).await
}