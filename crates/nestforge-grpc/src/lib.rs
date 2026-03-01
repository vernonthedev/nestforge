use std::{future::Future, net::SocketAddr, sync::Arc};

use anyhow::{Context, Result};
use nestforge_core::{
    framework_log_event, initialize_module_runtime, Container, InitializedModule, ModuleDefinition,
};
use tonic::Status;

pub use prost;
pub use tonic;

#[derive(Debug, Clone)]
pub struct GrpcServerConfig {
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
    pub fn new(addr: impl Into<String>) -> Self {
        Self { addr: addr.into() }
    }

    pub fn socket_addr(&self) -> Result<SocketAddr> {
        self.addr
            .parse()
            .with_context(|| format!("Invalid gRPC listen address `{}`", self.addr))
    }
}

#[derive(Clone)]
pub struct GrpcContext {
    container: Container,
}

impl GrpcContext {
    pub fn new(container: Container) -> Self {
        Self { container }
    }

    pub fn container(&self) -> &Container {
        &self.container
    }

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
