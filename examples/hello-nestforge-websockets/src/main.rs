use hello_nestforge_websockets::{AppModule, EventsGateway};
use nestforge::prelude::*;

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
