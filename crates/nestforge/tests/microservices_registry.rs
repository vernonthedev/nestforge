#![cfg(feature = "microservices")]

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use nestforge::MicroserviceClient;

#[derive(Clone)]
struct Counter(Arc<AtomicUsize>);

#[tokio::test]
async fn dispatches_message_patterns_with_contextual_dependency_resolution() {
    let container = nestforge::Container::new();
    container
        .register(Counter(Arc::new(AtomicUsize::new(41))))
        .expect("counter should register");

    let registry = nestforge::MicroserviceRegistry::builder()
        .message("users.count", |_payload: (), ctx| async move {
            let counter = ctx.resolve::<Counter>()?;
            Ok(counter.0.load(Ordering::Relaxed))
        })
        .build();

    let envelope = nestforge::MessageEnvelope::new("users.count", ()).expect("message");
    let ctx = nestforge::MicroserviceContext::new(
        container,
        "test",
        "users.count",
        nestforge::TransportMetadata::new(),
    );

    let response = registry
        .dispatch_message(envelope, ctx)
        .await
        .expect("response");

    assert_eq!(response, serde_json::json!(41));
}

#[tokio::test]
async fn dispatches_event_patterns_without_response_payloads() {
    let counter = Arc::new(AtomicUsize::new(0));
    let registry = nestforge::MicroserviceRegistry::builder()
        .event("users.created", {
            let counter = Arc::clone(&counter);
            move |_payload: serde_json::Value, _ctx| {
                let counter = Arc::clone(&counter);
                async move {
                    counter.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
            }
        })
        .build();

    let envelope = nestforge::EventEnvelope::new(
        "users.created",
        serde_json::json!({ "id": 1 }),
    )
    .expect("event");
    let ctx = nestforge::MicroserviceContext::new(
        nestforge::Container::new(),
        "test",
        "users.created",
        nestforge::TransportMetadata::new(),
    );

    registry
        .dispatch_event(envelope, ctx)
        .await
        .expect("event should dispatch");

    assert_eq!(counter.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn in_process_client_sends_and_deserializes_message_responses() {
    let container = nestforge::Container::new();
    let registry = nestforge::MicroserviceRegistry::builder()
        .message("users.count", |_payload: (), _ctx| async move { Ok(42usize) })
        .build();
    let client = nestforge::InProcessMicroserviceClient::new(container, registry)
        .with_transport("test-client");

    let response: usize = client
        .send("users.count", ())
        .await
        .expect("response should deserialize");

    assert_eq!(response, 42);
}

#[tokio::test]
async fn in_process_client_emits_events() {
    let counter = Arc::new(AtomicUsize::new(0));
    let registry = nestforge::MicroserviceRegistry::builder()
        .event("users.created", {
            let counter = Arc::clone(&counter);
            move |_payload: (), _ctx| {
                let counter = Arc::clone(&counter);
                async move {
                    counter.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
            }
        })
        .build();
    let client = nestforge::InProcessMicroserviceClient::new(
        nestforge::Container::new(),
        registry,
    );

    client
        .emit("users.created", ())
        .await
        .expect("event should dispatch");

    assert_eq!(counter.load(Ordering::Relaxed), 1);
}
