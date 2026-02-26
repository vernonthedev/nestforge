/*
Its job:
- load app modules/files
- boot the framework
*/

mod app_module;
mod app_config;
mod controllers;
mod guards;
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
    NestForgeFactory::<AppModule>::create()?
        .with_global_prefix("api")
        .use_guard::<AllowAllGuard>()
        .use_interceptor::<LoggingInterceptor>()
        .listen(PORT)
        .await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap().await
}
