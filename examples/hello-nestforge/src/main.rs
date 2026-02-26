/*
Its job:
- load app modules/files
- boot the framework
*/

mod app_module;
mod controllers;
mod dto;
mod services;

use app_module::AppModule;
use nestforge::NestForgeFactory;

const PORT: u16 = 3000;

async fn bootstrap() -> anyhow::Result<()> {
    NestForgeFactory::<AppModule>::create()?.listen(PORT).await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap().await
}
