# Quick Start

Get your first NestForge application up and running in minutes. This guide covers project creation, basic routing, and running the server.

## 1. Prerequisites

Ensure you have the Rust toolchain installed:

- [Install Rust](https://rustup.rs/)

## 2. Install NestForge CLI

The CLI is the recommended way to manage NestForge projects.

```bash
# Install from crates.io
cargo install nestforge-cli

# OR install from a local checkout (if developing the framework)
cargo install --path crates/nestforge-cli
```

## 3. Create a New Application

Scaffold a fresh HTTP project:

```bash
nestforge new my-nestforge-app
cd my-nestforge-app
```

The initial scaffold now creates a root barrel in `src/lib.rs` so the binary can import app
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

That means generated bootstrap code now looks more like:

```rust
use my_nestforge_app::AppModule;
use nestforge::prelude::*;
```

Feature modules can still use nested `controllers/`, `services/`, and `dto/` folders when you generate them that way.

## 4. Run the Server

NestForge projects use standard Cargo commands:

```bash
cargo run
```

By default, the server will be available at [http://127.0.0.1:3000](http://127.0.0.1:3000).

---

## Generator Layouts

NestForge supports two generator layouts:

- `nested`: controllers, services, and DTOs go into their own subfolders.
- `flat`: generated files stay side-by-side in the feature folder.

Use `--flat` when generating a module or resource:

```bash
nestforge g module users --flat
nestforge g resource users --module users --flat
```

When you run the resource generator in a terminal, NestForge can prompt for DTO fields and required/optional flags so the generated `Create*Dto`, `Update*Dto`, and entity DTO are usable immediately. Use `--no-prompt` if you want the default scaffold without interaction.

Flat layout output:

```text
src/users/
  mod.rs
  user_dto.rs
  create_user_dto.rs
  update_user_dto.rs
  users_controller.rs
  users_service.rs
```

Nested layout output:

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

If you prefer the older nested layout, just omit `--flat`.

---

## Basic Application Structure

A minimal NestForge app consists of an **AppModule** and a **Controller**.

### The Controller

Define your routes in a struct marked with `#[controller]`.

```rust
use nestforge::prelude::*;

#[controller("/")]
pub struct AppController;

#[routes]
impl AppController {
    #[nestforge::get("/")]
    async fn get_hello() -> ApiResult<String> {
        Ok(axum::Json("Hello from NestForge!".to_string()))
    }
}
```

### The Module

Wire everything together in a module.

```rust
use nestforge::prelude::*;
use crate::AppController;

#[module(
    controllers = [AppController],
    providers = [],
)]
pub struct AppModule;
```

### The Main Entry Point

Bootstrap the app using `NestForgeFactory`.

```rust
use my_nestforge_app::AppModule;
use nestforge::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    NestForgeFactory::<AppModule>::create()?
        .listen(3000)
        .await?;
    Ok(())
}
```

## Prelude and Barrel Imports

NestForge provides `nestforge::prelude::*` for the framework items most apps use repeatedly:

```rust
use nestforge::prelude::*;
```

Generated apps also add a root `src/lib.rs` barrel that re-exports top-level symbols such as
`AppModule` and `AppConfig`, so app code can stay flatter:

```rust
use my_nestforge_app::AppModule;
use my_nestforge_app::AppConfig;
```

---

## Next Steps

- **Add OpenAPI Documentation**: Learn how to [setup OpenAPI from scratch](./auth-openapi.md).
- **Export a Static Spec**: Run `nestforge export-docs --format yaml --output docs/openapi.yaml` once your app enables the `openapi` feature.
- **Generate Features**: Use `nestforge g module <name>` to add new features.
- **Use Flat Feature Layouts**: Pass `--flat` to keep generated controllers, services, and DTOs directly in the feature folder, for example `nestforge g resource users --module users --flat`.
- **Dependency Injection**: Explore the [Module System](./module-system.md).
