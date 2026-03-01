# gRPC

NestForge includes optional gRPC transport support through the `nestforge-grpc` crate.

## Enable The Feature

```toml
nestforge = { version = "1", features = ["grpc"] }
```

## Core Pieces

- `NestForgeGrpcFactory::<AppModule>`
- `GrpcServerConfig`
- `GrpcContext`
- re-exported `tonic` and `prost`

## Minimal Bootstrap

```rust
use nestforge::NestForgeGrpcFactory;

NestForgeGrpcFactory::<AppModule>::create()?
    .with_addr("127.0.0.1:50051")
    .listen_with(|ctx, addr| async move {
        tonic::transport::Server::builder()
            // .add_service(MyGeneratedServer::new(MyGrpcService::new(ctx)))
            .serve(addr)
            .await
    })
    .await?;
```

## Dependency Resolution Inside Services

`GrpcContext` gives generated tonic service implementations access to the NestForge container:

```rust
#[derive(Clone)]
struct GreeterService {
    ctx: nestforge::GrpcContext,
}

impl GreeterService {
    fn new(ctx: nestforge::GrpcContext) -> Self {
        Self { ctx }
    }
}
```

Then resolve shared providers as needed:

```rust
let config = self.ctx.resolve::<AppConfig>()?;
```

## Transport Shape

This setup mirrors the NestJS transport approach more than the HTTP controller approach:

- your module graph and providers still come from NestForge
- tonic-generated services remain the transport boundary
- `NestForgeGrpcFactory` wires DI and runtime address handling around that service layer
