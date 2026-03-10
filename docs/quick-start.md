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

## 4. Run the Server

NestForge projects use standard Cargo commands:

```bash
cargo run
```

By default, the server will be available at [http://127.0.0.1:3000](http://127.0.0.1:3000).

---

## Basic Application Structure

A minimal NestForge app consists of an **AppModule** and a **Controller**.

### The Controller

Define your routes in a struct marked with `#[controller]`.

```rust
use nestforge::{controller, routes, ApiResult};
use axum::Json;

#[controller("/")]
pub struct AppController;

#[routes]
impl AppController {
    #[nestforge::get("/")]
    async fn get_hello() -> ApiResult<String> {
        Ok(Json("Hello from NestForge!".to_string()))
    }
}
```

### The Module

Wire everything together in a module.

```rust
use nestforge::module;
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
use nestforge::NestForgeFactory;
use crate::app_module::AppModule;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    NestForgeFactory::<AppModule>::create()?
        .listen(3000)
        .await?;
    Ok(())
}
```

---

## Next Steps

- **Add OpenAPI Documentation**: Learn how to [setup OpenAPI from scratch](./auth-openapi.md).
- **Generate Features**: Use `nestforge g module <name>` to add new features.
- **Use Flat Feature Layouts**: Pass `--flat` to keep generated controllers, services, and DTOs directly in the feature folder, for example `nestforge g resource users --module users --flat`.
- **Dependency Injection**: Explore the [Module System](./module-system.md).
