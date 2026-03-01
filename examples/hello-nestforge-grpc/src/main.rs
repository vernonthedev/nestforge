mod app_config;
mod app_module;
mod grpc;

use app_module::AppModule;
use grpc::{proto::hello::greeter_server::GreeterServer, service::GreeterGrpcService};
use nestforge::NestForgeGrpcFactory;

const PORT: &str = "127.0.0.1:50051";

async fn bootstrap() -> anyhow::Result<()> {
    NestForgeGrpcFactory::<AppModule>::create()?
        .with_addr(PORT)
        .listen_with(|ctx, addr| async move {
            nestforge::tonic::transport::Server::builder()
                .add_service(GreeterServer::new(GreeterGrpcService::new(ctx)))
                .serve(addr)
                .await
        })
        .await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap().await
}
