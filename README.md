```text

███╗   ██╗███████╗███████╗████████╗███████╗ ██████╗ ██████╗  ██████╗ ███████╗
████╗  ██║██╔════╝██╔════╝╚══██╔══╝██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝
██╔██╗ ██║█████╗  ███████╗   ██║   █████╗  ██║   ██║██████╔╝██║  ███╗█████╗
██║╚██╗██║██╔══╝  ╚════██║   ██║   ██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝
██║ ╚████║███████╗███████║   ██║   ██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗
╚═╝  ╚═══╝╚══════╝╚══════╝   ╚═╝   ╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝

```

# NestForge

NestForge is a high-performance backend framework designed for developers who crave the modularity and **Dependency Injection (DI)** of NestJS but want the memory safety and blazing speed of the Rust ecosystem.

> [!IMPORTANT]
> **Stable Release**
> NestForge **1.0.0** is now published on crates.io.

## What You Get

- Module system with `imports` and `exports`
- Dependency Injection with simple provider registration
- Controller macros (`#[controller]`, `#[routes]`, `#[get]`, `#[post]`, `#[put]`, `#[delete]`)
- Request extractors (`Inject<T>`, `Param<T>`, `Body<T>`, `ValidatedBody<T>`)
- Extended request extractors (`Query<T>`, `Headers`, `Cookies`, `RequestId`)
- Built-in HTTP error type (`HttpException`)
- Guard and interceptor pipeline (global + route-level)
- Route-targeted middleware consumer API on the HTTP factory
- Auth primitives (`AuthUser`, `OptionalAuthUser`, `BearerToken`, auth resolvers, auth guards)
- Route versioning (`#[nestforge::version("1")]`)
- Global prefix support (`.with_global_prefix("api")`)
- Generated OpenAPI docs from controller metadata with runtime mounting helpers
- Optional GraphQL support through a dedicated `nestforge-graphql` crate and factory helpers
- Optional gRPC transport support through a dedicated `nestforge-grpc` crate
- Optional WebSocket gateway support through a dedicated `nestforge-websockets` crate
- Optional scheduler support through a dedicated `nestforge-schedule` crate
- Config module with env loading and schema validation
- Data layer crates (`nestforge-db`, `nestforge-orm`, `nestforge-data`)
- CLI for scaffolding, generators, DB migrations, docs skeleton, formatting

## Workspace Layout

- `crates/nestforge`: public crate users import
- `crates/nestforge-core`: DI, module graph, route builder, validation, resource service
- `crates/nestforge-http`: app bootstrap factory
- `crates/nestforge-macros`: framework macros
- `crates/nestforge-cli`: `nestforge` CLI binary
- `crates/nestforge-config`: env/config loading and validation
- `crates/nestforge-db`: DB wrapper and migrations support
- `crates/nestforge-orm`: relational ORM abstraction layer
- `crates/nestforge-data`: non-relational data abstractions
- `examples/hello-nestforge`: full example app
- `examples/hello-nestforge-graphql`: GraphQL-first example app
- `examples/hello-nestforge-grpc`: gRPC-first example app
- `examples/hello-nestforge-websockets`: WebSocket-first example app

## Quick Start (Repo)

```bash
git clone https://github.com/vernonthedev/nestforge.git
cd nestforge
cargo check --workspace
cargo run -p hello-nestforge
```

GraphQL-first example:

```bash
cargo run -p hello-nestforge-graphql
```

gRPC-first example:

```bash
cargo run -p hello-nestforge-grpc
```

WebSocket-first example:

```bash
cargo run -p hello-nestforge-websockets
```

Server runs on:

```text
http://127.0.0.1:3000
```

## Quick Start (CLI)

Install locally from this workspace:

```bash
cargo install --path crates/nestforge-cli
```

Create an app:

```bash
nestforge new demo-api
cd demo-api
cargo run
```

Create a GraphQL-first app:

```bash
nestforge new demo-graphql --transport graphql
```

Create a gRPC-first app:

```bash
nestforge new demo-grpc --transport grpc
```

Create a WebSocket-first app:

```bash
nestforge new demo-events --transport websockets
```

Generate code:

```bash
nestforge g module users
nestforge g resource users --module users
nestforge g guard auth
nestforge g interceptor logging
nestforge g graphql users
nestforge g grpc billing
```

DB commands:

```bash
nestforge db init
nestforge db generate create_users_table
nestforge db migrate
nestforge db status
```

Utilities:

```bash
nestforge docs
nestforge fmt
```

## Minimal App Bootstrap

```rust
use nestforge::NestForgeFactory;

NestForgeFactory::<AppModule>::create()?
    .with_global_prefix("api")
    .with_openapi_docs("My API", "1.0.0")?
    .use_guard::<AllowAllGuard>()
    .use_interceptor::<LoggingInterceptor>()
    .listen(3000)
    .await?;
```

## Example App Features

`examples/hello-nestforge` demonstrates:

- Root controllers (`AppController`, `HealthController`) at app root
- Feature modules (`users`, `settings`, `versioning`) in Nest-style folders
- CRUD controllers + services with `ResourceService<T>`
- Validation via `ValidatedBody<T>`
- Guard/interceptor usage at route level
- Generated `/docs` and `/openapi.json` routes from controller metadata
- Config loading with `ConfigModule::for_root`
- Versioned routes (`v1`, `v2`)

## Optional GraphQL Setup

Enable the `graphql` feature and merge a GraphQL schema directly into the app:

```rust
use nestforge::{
    async_graphql::{EmptyMutation, EmptySubscription, Object, Schema},
    NestForgeFactory, NestForgeFactoryGraphQlExt,
};

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn health(&self) -> &str {
        "ok"
    }
}

let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

NestForgeFactory::<AppModule>::create()?
    .with_graphql(schema)
    .listen(3000)
    .await?;
```

## Optional gRPC Setup

Enable the `grpc` feature and bootstrap a gRPC transport with the dedicated factory:

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

See `examples/hello-nestforge-grpc` for a full tonic-based setup with `proto/greeter.proto`,
`build.rs`, generated bindings, and provider resolution through `GrpcContext`.

## Optional WebSocket Setup

Enable the `websockets` feature and mount a gateway directly into the HTTP app:

```rust
use nestforge::{
    Message, NestForgeFactory, NestForgeFactoryWebSocketExt, WebSocket, WebSocketContext,
    WebSocketGateway,
};

struct EventsGateway;

impl WebSocketGateway for EventsGateway {
    fn on_connect(
        &self,
        _ctx: WebSocketContext,
        mut socket: WebSocket,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send>> {
        Box::pin(async move {
            let _ = socket.send(Message::Text("connected".into())).await;
        })
    }
}

NestForgeFactory::<AppModule>::create()?
    .with_websocket_gateway(EventsGateway)
    .listen(3000)
    .await?;
```

## Documentation

- Wiki: [https://github.com/vernonthedev/nestforge/wiki](https://github.com/vernonthedev/nestforge/wiki)
- Project docs: [https://vernonthedev.github.io/nestforge/docs/Home.md](https://vernonthedev.github.io/nestforge/docs/Home.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Apache-2.0 ([LICENSE](LICENSE)).
