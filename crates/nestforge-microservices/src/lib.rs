use std::{
    collections::{BTreeMap, HashMap},
    future::Future,
    pin::Pin,
    sync::Arc,
};

use anyhow::{Context, Result};
use nestforge_core::{AuthIdentity, Container, RequestId};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

type MessageFuture = Pin<Box<dyn Future<Output = Result<Value>> + Send>>;
type EventFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

pub trait MicroserviceClient: Send + Sync + 'static {
    fn send<Payload, Response>(
        &self,
        pattern: impl Into<String>,
        payload: Payload,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>>
    where
        Payload: Serialize + Send + 'static,
        Response: DeserializeOwned + Send + 'static;

    fn emit<Payload>(
        &self,
        pattern: impl Into<String>,
        payload: Payload,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>
    where
        Payload: Serialize + Send + 'static;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransportMetadata {
    pub values: BTreeMap<String, String>,
}

impl TransportMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.values.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    pub pattern: String,
    pub payload: Value,
    #[serde(default)]
    pub metadata: TransportMetadata,
}

impl MessageEnvelope {
    pub fn new(pattern: impl Into<String>, payload: impl Serialize) -> Result<Self> {
        Ok(Self {
            pattern: pattern.into(),
            payload: serde_json::to_value(payload)
                .context("Failed to serialize message payload")?,
            metadata: TransportMetadata::default(),
        })
    }

    pub fn with_metadata(mut self, metadata: TransportMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub pattern: String,
    pub payload: Value,
    #[serde(default)]
    pub metadata: TransportMetadata,
}

impl EventEnvelope {
    pub fn new(pattern: impl Into<String>, payload: impl Serialize) -> Result<Self> {
        Ok(Self {
            pattern: pattern.into(),
            payload: serde_json::to_value(payload).context("Failed to serialize event payload")?,
            metadata: TransportMetadata::default(),
        })
    }

    pub fn with_metadata(mut self, metadata: TransportMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Clone)]
pub struct MicroserviceContext {
    container: Container,
    transport: Arc<str>,
    pattern: Arc<str>,
    metadata: TransportMetadata,
    request_id: Option<RequestId>,
    auth_identity: Option<AuthIdentity>,
}

impl MicroserviceContext {
    pub fn new(
        container: Container,
        transport: impl Into<String>,
        pattern: impl Into<String>,
        metadata: TransportMetadata,
    ) -> Self {
        let request_id = container
            .resolve::<RequestId>()
            .ok()
            .map(|value| (*value).clone());
        let auth_identity = container
            .resolve::<AuthIdentity>()
            .ok()
            .map(|value| (*value).clone());

        Self {
            container,
            transport: Arc::<str>::from(transport.into()),
            pattern: Arc::<str>::from(pattern.into()),
            metadata,
            request_id,
            auth_identity,
        }
    }

    pub fn container(&self) -> &Container {
        &self.container
    }

    pub fn transport(&self) -> &str {
        &self.transport
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    pub fn metadata(&self) -> &TransportMetadata {
        &self.metadata
    }

    pub fn request_id(&self) -> Option<&RequestId> {
        self.request_id.as_ref()
    }

    pub fn auth_identity(&self) -> Option<&AuthIdentity> {
        self.auth_identity.as_ref()
    }

    pub fn resolve<T>(&self) -> Result<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.container.resolve::<T>().map_err(anyhow::Error::new)
    }
}

trait MessageHandler: Send + Sync + 'static {
    fn handle(&self, payload: Value, ctx: MicroserviceContext) -> MessageFuture;
}

trait EventHandler: Send + Sync + 'static {
    fn handle(&self, payload: Value, ctx: MicroserviceContext) -> EventFuture;
}

struct TypedMessageHandler<Payload, Response, Handler, Fut> {
    handler: Handler,
    _marker: std::marker::PhantomData<fn(Payload, Response, Fut)>,
}

impl<Payload, Response, Handler, Fut> MessageHandler
    for TypedMessageHandler<Payload, Response, Handler, Fut>
where
    Payload: DeserializeOwned + Send + 'static,
    Response: Serialize + Send + 'static,
    Handler: Fn(Payload, MicroserviceContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Response>> + Send + 'static,
{
    fn handle(&self, payload: Value, ctx: MicroserviceContext) -> MessageFuture {
        let payload = serde_json::from_value::<Payload>(payload)
            .context("Failed to deserialize message payload");

        match payload {
            Ok(payload) => {
                let future = (self.handler)(payload, ctx);
                Box::pin(async move {
                    let response = future.await?;
                    serde_json::to_value(response).context("Failed to serialize message response")
                })
            }
            Err(err) => Box::pin(async move { Err(err) }),
        }
    }
}

struct TypedEventHandler<Payload, Handler, Fut> {
    handler: Handler,
    _marker: std::marker::PhantomData<fn(Payload, Fut)>,
}

impl<Payload, Handler, Fut> EventHandler for TypedEventHandler<Payload, Handler, Fut>
where
    Payload: DeserializeOwned + Send + 'static,
    Handler: Fn(Payload, MicroserviceContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    fn handle(&self, payload: Value, ctx: MicroserviceContext) -> EventFuture {
        let payload = serde_json::from_value::<Payload>(payload)
            .context("Failed to deserialize event payload");

        match payload {
            Ok(payload) => Box::pin((self.handler)(payload, ctx)),
            Err(err) => Box::pin(async move { Err(err) }),
        }
    }
}

#[derive(Clone, Default)]
pub struct MicroserviceRegistry {
    message_handlers: Arc<HashMap<String, Arc<dyn MessageHandler>>>,
    event_handlers: Arc<HashMap<String, Arc<dyn EventHandler>>>,
}

impl MicroserviceRegistry {
    pub fn builder() -> MicroserviceRegistryBuilder {
        MicroserviceRegistryBuilder::default()
    }

    pub async fn dispatch_message(
        &self,
        envelope: MessageEnvelope,
        ctx: MicroserviceContext,
    ) -> Result<Value> {
        let handler = self
            .message_handlers
            .get(&envelope.pattern)
            .with_context(|| format!("No message handler registered for `{}`", envelope.pattern))?;

        handler.handle(envelope.payload, ctx).await
    }

    pub async fn dispatch_event(
        &self,
        envelope: EventEnvelope,
        ctx: MicroserviceContext,
    ) -> Result<()> {
        let handler = self
            .event_handlers
            .get(&envelope.pattern)
            .with_context(|| format!("No event handler registered for `{}`", envelope.pattern))?;

        handler.handle(envelope.payload, ctx).await
    }
}

#[derive(Default)]
pub struct MicroserviceRegistryBuilder {
    message_handlers: HashMap<String, Arc<dyn MessageHandler>>,
    event_handlers: HashMap<String, Arc<dyn EventHandler>>,
}

impl MicroserviceRegistryBuilder {
    pub fn message<Payload, Response, Handler, Fut>(
        mut self,
        pattern: impl Into<String>,
        handler: Handler,
    ) -> Self
    where
        Payload: DeserializeOwned + Send + 'static,
        Response: Serialize + Send + 'static,
        Handler: Fn(Payload, MicroserviceContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response>> + Send + 'static,
    {
        self.message_handlers.insert(
            pattern.into(),
            Arc::new(TypedMessageHandler::<Payload, Response, Handler, Fut> {
                handler,
                _marker: std::marker::PhantomData,
            }),
        );
        self
    }

    pub fn event<Payload, Handler, Fut>(
        mut self,
        pattern: impl Into<String>,
        handler: Handler,
    ) -> Self
    where
        Payload: DeserializeOwned + Send + 'static,
        Handler: Fn(Payload, MicroserviceContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.event_handlers.insert(
            pattern.into(),
            Arc::new(TypedEventHandler::<Payload, Handler, Fut> {
                handler,
                _marker: std::marker::PhantomData,
            }),
        );
        self
    }

    pub fn build(self) -> MicroserviceRegistry {
        MicroserviceRegistry {
            message_handlers: Arc::new(self.message_handlers),
            event_handlers: Arc::new(self.event_handlers),
        }
    }
}

#[derive(Clone)]
pub struct InProcessMicroserviceClient {
    container: Container,
    registry: MicroserviceRegistry,
    transport: Arc<str>,
    metadata: TransportMetadata,
}

impl InProcessMicroserviceClient {
    pub fn new(container: Container, registry: MicroserviceRegistry) -> Self {
        Self {
            container,
            registry,
            transport: Arc::<str>::from("in-process"),
            metadata: TransportMetadata::default(),
        }
    }

    pub fn with_transport(mut self, transport: impl Into<String>) -> Self {
        self.transport = Arc::<str>::from(transport.into());
        self
    }

    pub fn with_metadata(mut self, metadata: TransportMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    fn context(&self, pattern: impl Into<String>) -> MicroserviceContext {
        MicroserviceContext::new(
            self.container.clone(),
            self.transport.to_string(),
            pattern,
            self.metadata.clone(),
        )
    }
}

impl MicroserviceClient for InProcessMicroserviceClient {
    fn send<Payload, Response>(
        &self,
        pattern: impl Into<String>,
        payload: Payload,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>>
    where
        Payload: Serialize + Send + 'static,
        Response: DeserializeOwned + Send + 'static,
    {
        let registry = self.registry.clone();
        let pattern = pattern.into();
        let envelope = MessageEnvelope::new(pattern.clone(), payload);
        let context = self.context(pattern);

        Box::pin(async move {
            let response = registry.dispatch_message(envelope?, context).await?;
            serde_json::from_value(response).context("Failed to deserialize microservice response")
        })
    }

    fn emit<Payload>(
        &self,
        pattern: impl Into<String>,
        payload: Payload,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>
    where
        Payload: Serialize + Send + 'static,
    {
        let registry = self.registry.clone();
        let pattern = pattern.into();
        let envelope = EventEnvelope::new(pattern.clone(), payload);
        let context = self.context(pattern);

        Box::pin(async move { registry.dispatch_event(envelope?, context).await })
    }
}
