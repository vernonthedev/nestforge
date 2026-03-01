# WebSockets

NestForge can expose WebSocket gateways through the optional `nestforge-websockets` crate.

## Enable The Feature

```toml
nestforge = { version = "1", features = ["websockets"] }
```

## Gateway Setup

`WebSocketGateway` is the transport entry point. It receives a `WebSocketContext` and the upgraded `WebSocket`.

```rust
use nestforge::{
    Message, NestForgeFactory, NestForgeFactoryWebSocketExt, WebSocket, WebSocketContext,
    WebSocketGateway,
};

struct EventsGateway;

impl WebSocketGateway for EventsGateway {
    fn on_connect(
        &self,
        ctx: WebSocketContext,
        mut socket: WebSocket,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send>> {
        Box::pin(async move {
            if let Some(request_id) = ctx.request_id() {
                let _ = socket
                    .send(Message::Text(format!("connected:{}", request_id.value()).into()))
                    .await;
            }
        })
    }
}

NestForgeFactory::<AppModule>::create()?
    .with_websocket_gateway(EventsGateway)
    .listen(3000)
    .await?;
```

Default route:

- `/ws`

## Custom Routes

```rust
use nestforge::{websocket_gateway_router_with_config, WebSocketConfig};

let router = websocket_gateway_router_with_config(
    EventsGateway,
    WebSocketConfig::new("/events"),
);
```

That router can also be mounted manually with `NestForgeFactory::merge_router(...)`.

## Access To NestForge Providers

`WebSocketContext` keeps the framework `Container`, request id, auth identity, and request headers.

```rust
let service = ctx.resolve::<ChatService>()?;
```

This makes WebSocket gateways consistent with the DI flow already used by HTTP handlers, GraphQL resolvers, and gRPC services.
