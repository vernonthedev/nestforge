use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
};

use anyhow::Result;
use axum::{
    extract::Extension,
    http::HeaderMap,
    routing::get,
    Router,
};
use nestforge_core::{AuthIdentity, Container, RequestId};

pub use axum::extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket, WebSocketUpgrade};

type WebSocketFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

pub trait WebSocketGateway: Send + Sync + 'static {
    fn on_connect(&self, ctx: WebSocketContext, socket: WebSocket) -> WebSocketFuture;
}

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
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: normalize_path(endpoint.into(), "/ws"),
        }
    }
}

#[derive(Clone)]
pub struct WebSocketContext {
    container: Container,
    request_id: Option<RequestId>,
    auth_identity: Option<AuthIdentity>,
    headers: HeaderMap,
}

impl WebSocketContext {
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

    pub fn container(&self) -> &Container {
        &self.container
    }

    pub fn request_id(&self) -> Option<&RequestId> {
        self.request_id.as_ref()
    }

    pub fn auth_identity(&self) -> Option<&AuthIdentity> {
        self.auth_identity.as_ref()
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn resolve<T>(&self) -> Result<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.container.resolve::<T>()
    }

    pub fn is_authenticated(&self) -> bool {
        self.auth_identity.is_some()
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.auth_identity
            .as_ref()
            .map(|identity| identity.roles.iter().any(|value| value == role))
            .unwrap_or(false)
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
        get(move |ws: WebSocketUpgrade,
                  Extension(container): Extension<Container>,
                  headers: HeaderMap,
                  request_id: Option<RequestId>| {
                let gateway = Arc::clone(&gateway);
                let auth_identity = container
                    .resolve::<AuthIdentity>()
                    .ok()
                    .map(|value| (*value).clone());
                async move {
                    let context =
                        WebSocketContext::new(container, request_id, auth_identity, headers);
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
        get(move |ws: WebSocketUpgrade,
                  Extension(container): Extension<Container>,
                  headers: HeaderMap,
                  request_id: Option<RequestId>| {
                let handler = handler.clone();
                let auth_identity = container
                    .resolve::<AuthIdentity>()
                    .ok()
                    .map(|value| (*value).clone());
                async move {
                    let context =
                        WebSocketContext::new(container, request_id, auth_identity, headers);
                    ws.on_upgrade(move |socket| handler(context, socket))
                }
            }),
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
    use super::WebSocketConfig;

    #[test]
    fn config_normalizes_relative_paths() {
        assert_eq!(WebSocketConfig::new("socket").endpoint, "/socket");
        assert_eq!(WebSocketConfig::new("/socket").endpoint, "/socket");
        assert_eq!(WebSocketConfig::new("").endpoint, "/ws");
    }
}
