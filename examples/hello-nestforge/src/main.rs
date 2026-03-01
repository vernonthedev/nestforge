/*
Its job:
- load app modules/files
- boot the framework
*/

mod app_module;
mod app_config;
mod app_controller;
mod guards;
mod health_controller;
mod interceptors;
mod settings;
mod users;
mod versioning;

use app_module::AppModule;
use guards::AllowAllGuard;
use interceptors::LoggingInterceptor;
use nestforge::NestForgeFactory;

const PORT: u16 = 3000;

async fn bootstrap() -> anyhow::Result<()> {
    let docs_router = nestforge::openapi_docs_router_for_module::<AppModule>(
        "Hello NestForge API",
        "1.0.0",
    )?;

    NestForgeFactory::<AppModule>::create()?
        .with_global_prefix("api")
        .merge_router(docs_router)
        .use_guard::<AllowAllGuard>()
        .use_interceptor::<LoggingInterceptor>()
        .listen(PORT)
        .await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap().await
}
