use std::{future::Future, pin::Pin, sync::Arc};

use anyhow::Result;
use axum::{extract::Extension, http::HeaderMap, routing::get, Router};
use nestforge_core::{AuthIdentity, Container, RequestId};
use nestforge_microservices::{
    EventEnvelope, MessageEnvelope, MicroserviceContext, MicroserviceRegistry, TransportMetadata,
};
use serde_json::Value;

/**
 * Re-exports WebSocket types from axum for public use.
 *
 * These types provide the core WebSocket functionality for handling
 * real-time bidirectional communication.
 */
pub use axum::extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket, WebSocketUpgrade};

type WebSocketFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

/**
 * WebSocketGateway Trait
 *
 * The primary interface for implementing WebSocket gateways in NestForge.
 * Implement this trait to handle WebSocket connections with full access
 * to the NestForge container and authentication context.
 *
 * # Method
 * - `on_connect`: Called when a new WebSocket connection is established.
 *   Receives a WebSocketContext with DI container, request ID, auth identity,
 *   and headers, plus the WebSocket stream for communication.
 *
 * # Example
 * ```rust
 * struct MyGateway;
 *
 * impl WebSocketGateway for MyGateway {
 *     fn on_connect(&self, ctx: WebSocketContext, socket: WebSocket) -> WebSocketFuture {
 *         Box::pin(async move {
 *             // Handle WebSocket communication
 *             socket.close(None).await.ok();
 *         })
 *     }
 * }
 * ```
 */
pub trait WebSocketGateway: Send + Sync + 'static {
    fn on_connect(&self, ctx: WebSocketContext, socket: WebSocket) -> WebSocketFuture;
}

/**
 * WebSocketConfig
 *
 * Configuration options for WebSocket endpoints.
 *
 * # Fields
 * - `endpoint`: The URL path where the WebSocket handler is mounted
 *
 * # Defaults
 * - Endpoint defaults to "/ws"
 */
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub endpoint: String,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            endpoint: "/ws".to_string(),
        }
    }
}

impl WebSocketConfig {
    /**
     * Creates a new config with a custom endpoint.
     *
     * # Arguments
     * - `endpoint`: The URL path for the WebSocket endpoint
     */
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: normalize_path(endpoint.into(), "/ws"),
        }
    }
}

/**
 * WebSocketContext
 *
 * The context object provided to WebSocket handlers, containing
 * access to the DI container, request metadata, and authentication.
 *
 * # Access To
 * - DI Container for resolving services
 * - Request ID for logging/tracing
 * - Auth identity (if authenticated)
 * - HTTP headers
 *
 * # Usage in Handlers
 * ```rust
 * async fn handle_message(ctx: WebSocketContext, msg: Message) {
 *     let service = ctx.resolve::<MyService>().unwrap();
 *     // Use service...
 * }
 * ```
 */
#[derive(Clone)]
pub struct WebSocketContext {
    container: Container,
    request_id: Option<RequestId>,
    auth_identity: Option<AuthIdentity>,
    headers: HeaderMap,
}

impl WebSocketContext {
    /**
     * Creates a new WebSocket context.
     *
     * # Arguments
     * - `container`: The DI container for the request
     * - `request_id`: Optional request ID for tracing
     * - `auth_identity`: Optional authentication identity
     * - `headers`: HTTP headers from the request
     */
    pub fn new(
        container: Container,
        request_id: Option<RequestId>,
        auth_identity: Option<AuthIdentity>,
        headers: HeaderMap,
    ) -> Self {
        Self {
            container,
            request_id,
            auth_identity,
            headers,
        }
    }

    /**
     * Returns a reference to the DI container.
     */
    pub fn container(&self) -> &Container {
        &self.container
    }

    /**
     * Returns the request ID if available.
     */
    pub fn request_id(&self) -> Option<&RequestId> {
        self.request_id.as_ref()
    }

    /**
     * Returns the authentication identity if available.
     */
    pub fn auth_identity(&self) -> Option<&AuthIdentity> {
        self.auth_identity.as_ref()
    }

    /**
     * Returns a reference to the HTTP headers.
     */
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /**
     * Resolves a service from the DI container.
     *
     * # Type Parameters
     * - `T`: The type to resolve (must be Send + Sync + 'static)
     */
    pub fn resolve<T>(&self) -> Result<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        Ok(self.container.resolve::<T>()?)
    }

    /**
     * Checks if the WebSocket connection is authenticated.
     */
    pub fn is_authenticated(&self) -> bool {
        self.auth_identity.is_some()
    }

    /**
     * Checks if the authenticated user has a specific role.
     */
    pub fn has_role(&self, role: &str) -> bool {
        self.auth_identity
            .as_ref()
            .map(|identity| identity.roles.iter().any(|value| value == role))
            .unwrap_or(false)
    }

    /**
     * Creates a microservice context for dispatching messages over WebSocket.
     */
    pub fn microservice_context(
        &self,
        pattern: impl Into<String>,
        metadata: TransportMetadata,
    ) -> MicroserviceContext {
        MicroserviceContext::new(self.container.clone(), "websocket", pattern, metadata)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebSocketMicroserviceKind {
    Message,
    Event,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebSocketMicroserviceFrame {
    pub kind: WebSocketMicroserviceKind,
    pub pattern: String,
    pub payload: Value,
    #[serde(default)]
    pub metadata: TransportMetadata,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebSocketMicroserviceResponse {
    pub pattern: String,
    pub payload: Value,
    #[serde(default)]
    pub metadata: TransportMetadata,
}

pub async fn handle_websocket_microservice_message(
    ctx: &WebSocketContext,
    registry: &MicroserviceRegistry,
    message: Message,
) -> Result<Option<Message>> {
    let frame = parse_websocket_microservice_frame(message)?;
    let microservice_ctx = ctx.microservice_context(frame.pattern.clone(), frame.metadata.clone());

    match frame.kind {
        WebSocketMicroserviceKind::Message => {
            let payload = registry
                .dispatch_message(
                    MessageEnvelope {
                        pattern: frame.pattern.clone(),
                        payload: frame.payload,
                        metadata: frame.metadata.clone(),
                    },
                    microservice_ctx,
                )
                .await?;
            let response = WebSocketMicroserviceResponse {
                pattern: frame.pattern,
                payload,
                metadata: frame.metadata,
            };
            Ok(Some(Message::Text(
                serde_json::to_string(&response)?.into(),
            )))
        }
        WebSocketMicroserviceKind::Event => {
            registry
                .dispatch_event(
                    EventEnvelope {
                        pattern: frame.pattern,
                        payload: frame.payload,
                        metadata: frame.metadata,
                    },
                    microservice_ctx,
                )
                .await?;
            Ok(None)
        }
    }
}

fn parse_websocket_microservice_frame(message: Message) -> Result<WebSocketMicroserviceFrame> {
    match message {
        Message::Text(text) => Ok(serde_json::from_str(text.as_str())?),
        Message::Binary(bytes) => Ok(serde_json::from_slice(bytes.as_ref())?),
        other => anyhow::bail!("Unsupported websocket microservice message: {other:?}"),
    }
}

pub fn websocket_gateway_router<G>(gateway: G) -> Router<Container>
where
    G: WebSocketGateway,
{
    websocket_gateway_router_with_config(gateway, WebSocketConfig::default())
}

pub fn websocket_gateway_router_with_config<G>(
    gateway: G,
    config: WebSocketConfig,
) -> Router<Container>
where
    G: WebSocketGateway,
{
    let gateway = Arc::new(gateway);
    Router::new().route(
        &config.endpoint,
        get(
            move |ws: WebSocketUpgrade,
                  Extension(container): Extension<Container>,
                  headers: HeaderMap,
                  Extension(request_id): Extension<RequestId>| {
                let gateway = Arc::clone(&gateway);
                let auth_identity = container
                    .resolve::<AuthIdentity>()
                    .ok()
                    .map(|value| (*value).clone());
                async move {
                    let context =
                        WebSocketContext::new(container, Some(request_id), auth_identity, headers);
                    ws.on_upgrade(move |socket| async move {
                        gateway.on_connect(context, socket).await;
                    })
                }
            },
        ),
    )
}

pub fn websocket_router<F, Fut>(handler: F) -> Router<Container>
where
    F: Fn(WebSocketContext, WebSocket) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    websocket_router_with_config(handler, WebSocketConfig::default())
}

pub fn websocket_router_with_config<F, Fut>(
    handler: F,
    config: WebSocketConfig,
) -> Router<Container>
where
    F: Fn(WebSocketContext, WebSocket) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    Router::new().route(
        &config.endpoint,
        get(
            move |ws: WebSocketUpgrade,
                  Extension(container): Extension<Container>,
                  headers: HeaderMap,
                  Extension(request_id): Extension<RequestId>| {
                let handler = handler.clone();
                let auth_identity = container
                    .resolve::<AuthIdentity>()
                    .ok()
                    .map(|value| (*value).clone());
                async move {
                    let context =
                        WebSocketContext::new(container, Some(request_id), auth_identity, headers);
                    ws.on_upgrade(move |socket| handler(context, socket))
                }
            },
        ),
    )
}

fn normalize_path(path: String, fallback: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "/" {
        return fallback.to_string();
    }

    if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use axum::http::HeaderMap;
    use nestforge_microservices::MicroserviceRegistry;

    use super::{
        handle_websocket_microservice_message, Message, TransportMetadata, WebSocketConfig,
        WebSocketContext, WebSocketMicroserviceFrame, WebSocketMicroserviceKind,
    };

    #[test]
    fn config_normalizes_relative_paths() {
        assert_eq!(WebSocketConfig::new("socket").endpoint, "/socket");
        assert_eq!(WebSocketConfig::new("/socket").endpoint, "/socket");
        assert_eq!(WebSocketConfig::new("").endpoint, "/ws");
    }

    #[tokio::test]
    async fn websocket_microservice_adapter_returns_message_responses() {
        let container = nestforge_core::Container::new();
        container
            .register(Arc::new(AtomicUsize::new(7)))
            .expect("counter should register");
        let ctx = WebSocketContext::new(container, None, None, HeaderMap::new());
        let registry = MicroserviceRegistry::builder()
            .message("counter.read", |_payload: (), ctx| async move {
                let counter = ctx.resolve::<Arc<AtomicUsize>>()?;
                Ok(counter.load(Ordering::Relaxed))
            })
            .build();
        let frame = WebSocketMicroserviceFrame {
            kind: WebSocketMicroserviceKind::Message,
            pattern: "counter.read".to_string(),
            payload: serde_json::json!(null),
            metadata: TransportMetadata::default(),
        };

        let response = handle_websocket_microservice_message(
            &ctx,
            &registry,
            Message::Text(serde_json::to_string(&frame).expect("frame").into()),
        )
        .await
        .expect("message should dispatch");

        match response {
            Some(Message::Text(payload)) => {
                let json: serde_json::Value =
                    serde_json::from_str(payload.as_str()).expect("response should be json");
                assert_eq!(json["payload"], serde_json::json!(7));
            }
            other => panic!("unexpected websocket response: {other:?}"),
        }
    }

    #[tokio::test]
    async fn websocket_microservice_adapter_dispatches_events_without_response() {
        let counter = Arc::new(AtomicUsize::new(0));
        let ctx = WebSocketContext::new(
            nestforge_core::Container::new(),
            None,
            None,
            HeaderMap::new(),
        );
        let registry = MicroserviceRegistry::builder()
            .event("counter.bump", {
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
        let frame = WebSocketMicroserviceFrame {
            kind: WebSocketMicroserviceKind::Event,
            pattern: "counter.bump".to_string(),
            payload: serde_json::json!(null),
            metadata: TransportMetadata::default(),
        };

        let response = handle_websocket_microservice_message(
            &ctx,
            &registry,
            Message::Text(serde_json::to_string(&frame).expect("frame").into()),
        )
        .await
        .expect("event should dispatch");

        assert!(response.is_none());
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }
}
