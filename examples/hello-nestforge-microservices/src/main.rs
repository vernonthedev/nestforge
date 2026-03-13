mod app_config;
mod app_module;
mod microservices;

use std::sync::atomic::Ordering;

use app_module::AppModule;
use microservices::{AppPatterns, EventCounter, GreetingPayload};
use nestforge::{MicroserviceClient, TestFactory, TransportMetadata};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let module = TestFactory::<AppModule>::create().build()?;
    let patterns = module.resolve::<AppPatterns>()?;
    let client = module.microservice_client_with_metadata(
        patterns.registry().clone(),
        "example-cli",
        TransportMetadata::new().insert("example", "hello-nestforge-microservices"),
    );

    let greeting: serde_json::Value = client
        .send(
            "app.greet",
            GreetingPayload {
                name: "John Doe".to_string(),
            },
        )
        .await?;
    println!("{}", serde_json::to_string_pretty(&greeting)?);

    client.emit("app.bump", ()).await?;
    let counter = module.resolve::<EventCounter>()?;
    println!("events_processed={}", counter.0.load(Ordering::Relaxed));

    module.shutdown()?;
    Ok(())
}
