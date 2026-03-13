use std::{future::Future, net::SocketAddr, sync::Arc};

use anyhow::{Context, Result};
use nestforge_core::{
    framework_log_event, initialize_module_runtime, Container, InitializedModule, ModuleDefinition,
};
use nestforge_microservices::{
    EventEnvelope, MessageEnvelope, MicroserviceContext, MicroserviceRegistry, TransportMetadata,
};
use serde::Serialize;
use tonic::Status;

/** Re-exports prost for protobuf message handling */
pub use prost;
/** Re-exports tonic for gRPC server/client generation */
pub use tonic;

/**
 * GrpcServerConfig
 *
 * Configuration options for the gRPC server.
 */
#[derive(Debug, Clone)]
pub struct GrpcServerConfig {
    /** The address to listen on (host:port) */
    pub addr: String,
}

impl Default for GrpcServerConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:50051".to_string(),
        }
    }
}

impl GrpcServerConfig {
    /**
     * Creates a new GrpcServerConfig with the given address.
     */
    pub fn new(addr: impl Into<String>) -> Self {
        Self { addr: addr.into() }
    }

    /**
     * Parses the address into a SocketAddr.
     */
    pub fn socket_addr(&self) -> Result<SocketAddr> {
        self.addr
            .parse()
            .with_context(|| format!("Invalid gRPC listen address `{}`", self.addr))
    }
}

/**
 * GrpcContext
 *
 * The execution context for gRPC service handlers.
 * Provides access to the DI container and microservice dispatch.
 */
#[derive(Clone)]
pub struct GrpcContext {
    container: Container,
}

impl GrpcContext {
    /**
     * Creates a new GrpcContext with the given container.
     */
    pub fn new(container: Container) -> Self {
        Self { container }
    }

    /**
     * Returns a reference to the DI container.
     */
    pub fn container(&self) -> &Container {
        &self.container
    }

    /**
     * Resolves a service from the DI container.
     *
     * # Type Parameters
     * - `T`: The type to resolve (must be Send + Sync + 'static)
     */
    pub fn resolve<T>(&self) -> Result<Arc<T>, Status>
    where
        T: Send + Sync + 'static,
    {
        self.container.resolve::<T>().map_err(|err| {
            Status::internal(format!(
                "Failed to resolve dependency `{}`: {}",
                std::any::type_name::<T>(),
                err
            ))
        })
    }

    /**
     * Creates a microservice context for dispatching messages from gRPC.
     */
    pub fn microservice_context(
        &self,
        pattern: impl Into<String>,
        metadata: TransportMetadata,
    ) -> MicroserviceContext {
        MicroserviceContext::new(self.container.clone(), "grpc", pattern, metadata)
    }
}

/**
 * Dispatches a gRPC message to the microservice registry.
 *
 * # Type Parameters
 * - `Payload`: The message payload type
 * - `Response`: The response type
 *
 * # Arguments
 * - `ctx`: The gRPC context
 * - `registry`: The microservice registry
 * - `pattern`: The message pattern
 * - `payload`: The message payload
 */
pub async fn dispatch_grpc_message<Payload>(
    ctx: &GrpcContext,
    registry: &MicroserviceRegistry,
    pattern: impl Into<String>,
    payload: Payload,
    metadata: TransportMetadata,
) -> Result<serde_json::Value, Status>
where
    Payload: Serialize,
{
    let pattern = pattern.into();
    let envelope =
        MessageEnvelope::new(pattern.clone(), payload).map_err(map_microservice_error)?;
    let envelope = envelope.with_metadata(metadata.clone());
    let context = ctx.microservice_context(pattern, metadata);

    registry
        .dispatch_message(envelope, context)
        .await
        .map_err(map_microservice_error)
}

pub async fn dispatch_grpc_event<Payload>(
    ctx: &GrpcContext,
    registry: &MicroserviceRegistry,
    pattern: impl Into<String>,
    payload: Payload,
    metadata: TransportMetadata,
) -> Result<(), Status>
where
    Payload: Serialize,
{
    let pattern = pattern.into();
    let envelope = EventEnvelope::new(pattern.clone(), payload).map_err(map_microservice_error)?;
    let envelope = envelope.with_metadata(metadata.clone());
    let context = ctx.microservice_context(pattern, metadata);

    registry
        .dispatch_event(envelope, context)
        .await
        .map_err(map_microservice_error)
}

fn map_microservice_error(err: anyhow::Error) -> Status {
    Status::internal(err.to_string())
}

pub struct NestForgeGrpcFactory<M: ModuleDefinition> {
    _marker: std::marker::PhantomData<M>,
    container: Container,
    runtime: Arc<InitializedModule>,
    config: GrpcServerConfig,
}

impl<M: ModuleDefinition> NestForgeGrpcFactory<M> {
    pub fn create() -> Result<Self> {
        let container = Container::new();
        let runtime = Arc::new(initialize_module_runtime::<M>(&container)?);
        runtime.run_module_init(&container)?;
        runtime.run_application_bootstrap(&container)?;

        Ok(Self {
            _marker: std::marker::PhantomData,
            container,
            runtime,
            config: GrpcServerConfig::default(),
        })
    }

    pub fn with_addr(mut self, addr: impl Into<String>) -> Self {
        self.config = GrpcServerConfig::new(addr);
        self
    }

    pub fn with_config(mut self, config: GrpcServerConfig) -> Self {
        self.config = config;
        self
    }

    pub fn container(&self) -> &Container {
        &self.container
    }

    pub fn context(&self) -> GrpcContext {
        GrpcContext::new(self.container.clone())
    }

    pub fn socket_addr(&self) -> Result<SocketAddr> {
        self.config.socket_addr()
    }

    pub async fn listen_with<F, Fut, E>(self, serve: F) -> Result<()>
    where
        F: FnOnce(GrpcContext, SocketAddr) -> Fut,
        Fut: Future<Output = std::result::Result<(), E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let runtime = Arc::clone(&self.runtime);
        let container = self.container.clone();
        let addr = self.socket_addr()?;
        framework_log_event("grpc_server_listening", &[("addr", addr.to_string())]);
        serve(self.context(), addr)
            .await
            .map_err(anyhow::Error::new)
            .context("gRPC transport server failed")?;
        runtime.run_module_destroy(&container)?;
        runtime.run_application_shutdown(&container)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use nestforge_microservices::MicroserviceRegistry;

    use super::{dispatch_grpc_event, dispatch_grpc_message, GrpcContext, TransportMetadata};

    #[derive(Clone)]
    struct Counter(Arc<AtomicUsize>);

    #[tokio::test]
    async fn grpc_dispatch_adapter_invokes_message_registry() {
        let container = nestforge_core::Container::new();
        container
            .register(Counter(Arc::new(AtomicUsize::new(3))))
            .expect("counter should register");
        let ctx = GrpcContext::new(container);
        let registry = MicroserviceRegistry::builder()
            .message("counter.read", |_payload: (), ctx| async move {
                let counter = ctx.resolve::<Counter>()?;
                Ok(counter.0.load(Ordering::Relaxed))
            })
            .build();

        let response = dispatch_grpc_message(
            &ctx,
            &registry,
            "counter.read",
            (),
            TransportMetadata::new().insert("transport", "grpc"),
        )
        .await
        .expect("message should dispatch");

        assert_eq!(response, serde_json::json!(3));
    }

    #[tokio::test]
    async fn grpc_dispatch_adapter_invokes_event_registry() {
        let counter = Arc::new(AtomicUsize::new(0));
        let ctx = GrpcContext::new(nestforge_core::Container::new());
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

        dispatch_grpc_event(
            &ctx,
            &registry,
            "counter.bump",
            (),
            TransportMetadata::default(),
        )
        .await
        .expect("event should dispatch");

        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }
}
