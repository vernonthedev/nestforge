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

[![crates.io](https://img.shields.io/crates/v/nestforge?label=crates.io)](https://crates.io/crates/nestforge)
[![docs.rs](https://img.shields.io/docsrs/nestforge?label=docs.rs)](https://docs.rs/nestforge)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/actions/workflow/status/vernonthedev/nestforge/release.yml?branch=main&label=release)](https://github.com/vernonthedev/nestforge/actions/workflows/release.yml)
[![VS Code Extension](https://img.shields.io/visual-studio-marketplace/v/vernonthedev.nestforge?label=VS%20Code%20Extension)](https://marketplace.visualstudio.com/items?itemName=vernonthedev.nestforge)
[![Extension Downloads](https://img.shields.io/visual-studio-marketplace/d/vernonthedev.nestforge?label=extension%20downloads)](https://marketplace.visualstudio.com/items?itemName=vernonthedev.nestforge)

> [!IMPORTANT]
> **Stable Release**
> NestForge **1.6.0** is now published on crates.io.

## What You Get

- Module system with `imports` and `exports`
- Dependency Injection with simple provider registration
- Managed singleton services with `#[injectable]`
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
- Swagger UI and Redoc hosting for generated OpenAPI docs
- DTO-driven OpenAPI schema generation for request and response bodies
- Optional GraphQL support through a dedicated `nestforge-graphql` crate and factory helpers
- Optional gRPC transport support through a dedicated `nestforge-grpc` crate
- Optional WebSocket gateway support through a dedicated `nestforge-websockets` crate
- Optional scheduler support through a dedicated `nestforge-schedule` crate
- Config module with env loading and schema validation
- Data layer crates (`nestforge-db`, `nestforge-orm`, `nestforge-data`)
- Testing helpers with module overrides plus HTTP and GraphQL test routers
- CLI for scaffolding, generators, flat or nested feature layouts, DB migrations, docs skeleton, formatting

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
- `examples/hello-nestforge-microservices`: microservice registry + in-process client example app
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

Microservices-first example:

```bash
cargo run -p hello-nestforge-microservices
```

Server runs on:

```text
http://127.0.0.1:3000
```

## Releases

NestForge now uses a Rust-native direct release flow driven by the repository release script.

- Pushes to `main` run the repository release script directly.
- Conventional commits since the last version tag determine the next semver bump automatically.
- Changed crates are versioned, tagged, released on GitHub, and published to crates.io in dependency order.
- The primary published changelog is updated at `crates/nestforge/CHANGELOG.md`, so conventional commits remain the source for changelog entries.

Repository setup required for publishing:

- Add `CARGO_REGISTRY_TOKEN` to GitHub Actions secrets.
- Keep using Conventional Commits for changes you want included in the release notes.
- Expect first-time publishes for brand new crates to require a manual bootstrap publish before full automation can take over.

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

Scaffold a new HTTP app with OpenAPI docs wired in from day one:

```bash
nestforge new demo-api --openapi
```

The default app scaffold now creates a root `src/lib.rs` barrel so the binary can import app
symbols directly from the package crate:

```text
src/
  lib.rs
  main.rs
  app_config.rs
  app_controller.rs
  app_module.rs
  health_controller.rs
```

That means generated bootstrap code looks more like:

```rust
use demo_api::AppModule;
use nestforge::prelude::*;
```

instead of a long list of `mod app_config;`, `mod app_module;`, and similar declarations in
`main.rs`.

Nested `controllers/` and `services/` folders are only created later when you generate root-level resources into them.

Create a GraphQL-first app:

```bash
nestforge new demo-graphql --transport graphql
```

Create a gRPC-first app:

```bash
nestforge new demo-grpc --transport grpc
```

Create a microservices-first app:

```bash
nestforge new demo-bus --transport microservices
```

Create a WebSocket-first app:

```bash
nestforge new demo-events --transport websockets
```

Generate code:

```bash
nestforge g module users
nestforge g resource users --module users
nestforge g resource users --module users --flat
nestforge g guard auth
nestforge g filter rewrite_bad_request
nestforge g middleware audit
nestforge g interceptor logging
nestforge g graphql users
nestforge g grpc billing
nestforge g gateway events
```

Flat feature layout:

```bash
nestforge g module users --flat
nestforge g resource users --module users --flat
```

This keeps generated files together in the feature root:

```text
src/users/
  mod.rs
  user_dto.rs
  create_user_dto.rs
  update_user_dto.rs
  users_controller.rs
  users_service.rs
```

Without `--flat`, the CLI keeps the older nested layout:

```text
src/users/
  mod.rs
  controllers/
    users_controller.rs
  services/
    users_service.rs
  dto/
    user_dto.rs
    create_user_dto.rs
    update_user_dto.rs
```

When you generate a resource from a real terminal, the CLI can now prompt for DTO fields so the scaffolded `Create*Dto`, `Update*Dto`, and entity DTO match your domain instead of defaulting to a single `name` field. For non-interactive runs, pass `--no-prompt` or let the CLI fall back to the default field set.

```bash
nestforge g resource users --module users --flat
nestforge g resource users --module users --flat --no-prompt
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

## VS Code Extension

NestForge also ships with an official VS Code extension for driving the `nestforge` CLI directly from the editor.

- Marketplace: [vernonthedev.nestforge](https://marketplace.visualstudio.com/items?itemName=vernonthedev.nestforge)
- Repository: `vernonthedev/nestforge-extension`
- Best for guided scaffolding, generator workflows, DB commands, OpenAPI docs access, Rust utilities, onboarding, and module-aware Explorer actions

The extension currently provides:

- New application and generator wizards
- Explorer context menus for module-aware generation
- DB dashboard commands and status polling
- `.env` validation with quick fixes
- OpenAPI docs shortcuts, format commands, run config generation, Git setup, logs, and module graph views
- Rust snippets for common NestForge patterns

## Minimal App Bootstrap

```rust
use my_app::AppModule;
use nestforge::prelude::*;

NestForgeFactory::<AppModule>::create()?
    .with_global_prefix("api")
    .with_openapi_docs("My API", "1.0.0")?
    .use_guard::<AllowAllGuard>()
    .use_interceptor::<LoggingInterceptor>()
    .listen(3000)
    .await?;
```

## Prelude and Root Re-exports

NestForge includes a lightweight `nestforge::prelude` for the framework items you reach for most
often in app code and generated scaffolds:

```rust
use nestforge::prelude::*;
```

The prelude is meant to reduce repetitive framework import noise. It groups together the common
factories, route/module macros, and helper types used in everyday NestForge code.

Examples of what it gives you:

- `NestForgeFactory`
- `NestForgeGrpcFactory`
- `NestForgeFactoryGraphQlExt`
- `NestForgeFactoryWebSocketExt`
- `Inject`
- `HttpException`
- common route and module macros such as `#[module]`, `#[controller]`, and `#[get]`

On top of that, generated apps now include a root `src/lib.rs` barrel that re-exports top-level app
symbols like `AppModule` and `AppConfig`. That lets you write:

```rust
use demo_api::AppModule;
use demo_api::AppConfig;
```

instead of pushing those imports through `mod ...;` declarations or deeper nested paths in
`main.rs`.

## Injectable Services

Use `#[injectable]` when you want NestForge to treat a struct as a managed provider without a
manual `#[derive(Clone)]`.

```rust
use nestforge::{injectable, module, Inject};

#[injectable]
pub struct UsersService;

#[module(
    providers = [UsersService],
    exports = [UsersService]
)]
pub struct UsersModule;

#[nestforge::controller("/users")]
pub struct UsersController;

#[nestforge::routes]
impl UsersController {
    #[nestforge::get("/count")]
    async fn count(_users: Inject<UsersService>) -> Result<&'static str, nestforge::HttpException> {
        Ok("ok")
    }
}
```

If a service needs custom setup, point the macro at a zero-arg factory:

```rust
#[injectable(factory = build_users_service)]
pub struct UsersService {
    pub label: &'static str,
}

fn build_users_service() -> anyhow::Result<UsersService> {
    Ok(UsersService { label: "users" })
}
```

The same pattern works nicely for framework-managed config objects in the examples:

```rust
use nestforge::{injectable, ConfigModule, ConfigOptions};

#[injectable(factory = load_app_config)]
pub struct AppConfig {
    pub app_name: String,
}

fn load_app_config() -> anyhow::Result<AppConfig> {
    Ok(ConfigModule::for_root::<AppConfig>(
        ConfigOptions::new().env_file(".env"),
    )?)
}

#[nestforge::module(
    providers = [AppConfig],
    exports = [AppConfig]
)]
pub struct AppModule;
```

Keep explicit value or factory providers for external runtime resources like database connections,
clients, or other values you do not own as plain structs.

## Example App Features

`examples/hello-nestforge` demonstrates:

- Root controllers (`AppController`, `HealthController`) at app root
- Feature modules (`users`, `settings`, `versioning`) in Nest-style folders
- CLI generators also support flat feature folders with `--flat` when you want controllers, services, and DTOs side-by-side in the module root
- CRUD controllers + services with `ResourceService<T>`
- Validation via `ValidatedBody<T>`
- Guard/interceptor usage at route level
- Generated `/docs` and `/openapi.json` routes from controller metadata
- Generated OpenAPI schemas for DTOs used in `ValidatedBody<T>`, `Body<T>`, and `ApiResult<T>`

Export a static spec for CI/CD or frontend handoff:

```bash
nestforge export-docs
nestforge export-docs --format yaml --output docs/openapi.yaml
```

- Config loading with `ConfigModule::for_root` and `#[injectable(factory = ...)]`
- Versioned routes (`v1`, `v2`)

## Optional GraphQL Setup

Enable the `graphql` feature and merge a GraphQL schema directly into the app:

```rust
use nestforge::{async_graphql::{EmptyMutation, EmptySubscription, Object, Schema}, prelude::*};

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
use nestforge::prelude::*;

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
use nestforge::{prelude::*, Message, WebSocket, WebSocketContext, WebSocketGateway};

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

- Main Documentation: [https://nestforge.suredoc.net](https://nestforge.suredoc.net)
- Wiki: [https://github.com/vernonthedev/nestforge/wiki](https://github.com/vernonthedev/nestforge/wiki)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Apache-2.0 ([LICENSE](LICENSE)).
