# Microservices

NestForge now includes a transport-agnostic microservice registry for Nest-style message and event patterns.

This layer is intentionally transport-neutral:

- gRPC can map incoming RPCs into patterns
- WebSockets can map frames into patterns
- future brokers can map subjects or topics into patterns

## Registry

Build a registry with typed message and event handlers:

```rust
let registry = nestforge::MicroserviceRegistry::builder()
    .message("users.count", |_payload: (), ctx| async move {
        let service = ctx.resolve::<UsersService>()?;
        Ok(service.count().await?)
    })
    .event("users.created", |payload: CreateUserEvent, ctx| async move {
        let audit = ctx.resolve::<AuditService>()?;
        audit.record(payload.user_id).await?;
        Ok(())
    })
    .build();
```

## Dispatch

Transports can dispatch typed envelopes into that registry:

```rust
let envelope = nestforge::MessageEnvelope::new("users.count", ())?;
let ctx = nestforge::MicroserviceContext::new(
    container.clone(),
    "grpc",
    "users.count",
    nestforge::TransportMetadata::new(),
);

let response = registry.dispatch_message(envelope, ctx).await?;
```

## Client

`InProcessMicroserviceClient` lets app code and tests call patterns without manually building envelopes:

```rust
use nestforge::MicroserviceClient;

let client = nestforge::InProcessMicroserviceClient::new(container.clone(), registry.clone());
let count: usize = client.send("users.count", ()).await?;
client.emit("users.created", CreateUserEvent { user_id: 7 }).await?;
```

The `hello-nestforge-microservices` example shows this flow end to end with a DI-backed registry and client.

## Context

`MicroserviceContext` exposes:

- the framework container
- the current transport name
- the resolved pattern
- transport metadata
- optional request id and auth identity when present in the container
