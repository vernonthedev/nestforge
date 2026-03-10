/*
Its job:
- load app modules/files
- boot the framework
*/

mod app_config;
mod app_controller;
mod app_module;
mod guards;
mod health_controller;
mod interceptors;
mod serializers;
mod settings;
mod users;
mod versioning;

use app_module::AppModule;
use guards::AllowAllGuard;
use interceptors::LoggingInterceptor;
use nestforge::{NestForgeFactory, NestForgeFactoryOpenApiExt};

const PORT: u16 = 3000;

async fn bootstrap() -> anyhow::Result<()> {
    NestForgeFactory::<AppModule>::create()?
        .with_global_prefix("api")
        .with_openapi_docs("Hello NestForge API", "1.0.0")?
        .use_guard::<AllowAllGuard>()
        .use_interceptor::<LoggingInterceptor>()
        .listen(PORT)
        .await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap().await
}
