mod app_config;
mod app_module;
mod ws;

use app_module::AppModule;
use nestforge::{NestForgeFactory, NestForgeFactoryWebSocketExt};
use ws::EventsGateway;

const PORT: u16 = 3002;

async fn bootstrap() -> anyhow::Result<()> {
    NestForgeFactory::<AppModule>::create()?
        .with_websocket_gateway(EventsGateway)
        .listen(PORT)
        .await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap().await
}
