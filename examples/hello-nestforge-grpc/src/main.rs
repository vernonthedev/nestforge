use hello_nestforge_grpc::{proto::hello::greeter_server::GreeterServer, AppModule, GreeterGrpcService};
use nestforge::prelude::*;

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
