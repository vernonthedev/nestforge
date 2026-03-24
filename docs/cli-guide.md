# CLI Guide

## Install

```bash
cargo install --path crates/nestforge-cli
```

## Command Overview

```text
nestforge new <app-name>
nestforge new <app-name> --transport <http|graphql|grpc|microservices|websockets>
nestforge start [app-name]
nestforge dev [app-name]
nestforge g module <name>
nestforge g resource <name>
nestforge g controller <name>
nestforge g service <name>
nestforge g guard <name>
nestforge g decorator <name>
nestforge g filter <name>
nestforge g middleware <name>
nestforge g interceptor <name>
nestforge g serializer <name>
nestforge g graphql <name>
nestforge g grpc <name>
nestforge g gateway <name>
nestforge g microservice <name>
nestforge db init
nestforge db generate <name>
nestforge db migrate
nestforge db status
nestforge docs
nestforge fmt
```

## Running Your App

### nestforge start

Runs your application with automatic TypeScript-style import transpilation:

```bash
nestforge start
```

This command:
1. Scans your `src/` directory for files containing `import` statements
2. Transpiles TypeScript-style imports to valid Rust `use` statements
3. Writes transpiled code to `.nestforge/cache/`
4. Runs `cargo run` to execute your application

### nestforge dev

Runs your application in development mode:

```bash
nestforge dev
```

Same as `start` but optimized for development workflow.

### TypeScript-Style Imports

NestForge supports TypeScript-style import syntax that gets transpiled to Rust:

```typescript
// Your code (src/users/users.controller.ts)
import { Controller, Get } from "nestforge/common";
import { UsersService } from "./users.service";

@Controller("users")
export class UsersController {
    @Get()
    findAll() {
        return [];
    }
}
```

Transpiles to:

```rust
// Transpiled (.nestforge/cache/users/users_controller.rs)
use nestforge::common::{Controller, Get};
use self::users_service::UsersService;

#[Controller("users")]
pub struct UsersController {
    #[Get()]
    pub fn find_all(&self) -> Vec<()> {
        Vec::new()
    }
}
```

#### Supported Import Patterns

| Pattern | Example | Transpiles To |
|---------|---------|---------------|
| Named import | `import { Module, Controller } from "nestforge/common"` | `use nestforge::common::{Module, Controller}` |
| NestForge import | `import { Get, Post } from "nestforge/common"` | `use nestforge::common::{Get, Post}` |
| Relative import | `import { UsersService } from "./users.service"` | `use self::users_service::UsersService` |
| Parent import | `import { Config } from "../config"` | `use super::config::Config` |
| Default import | `import MyService from "./my.service"` | `use self::my_service::MyService` |

#### Path Transformations

- `nestforge/common` → `nestforge::common`
- `nestforge/http` → `nestforge::http`
- `./users.service` → `self::users_service`
- `../config` → `super::config`
- `../../shared/utils` → `super::super::shared::utils`

#### Case Conversion

The transpiler automatically converts filenames to snake_case:

- `users.service` → `users_service`
- `authController` → `auth_controller`
- `my-controller.ts` → `my_controller`

#### Caveats

- Standard `cargo run` will fail on files using `import` syntax
- Always use `nestforge start` or `nestforge dev` to run your app
- The transpiler writes to `.nestforge/cache/` - do not edit these files directly

## Generators

### New App

```bash
nestforge new demo-api
```

Creates a runnable app with:

- `lib.rs` root barrel with app re-exports
- `main.rs`
- `app_module.rs`
- root app/health controllers
- guards/interceptors folders
- `.env` and `.env.example`

Generate a GraphQL-first app:

```bash
nestforge new demo-graphql --transport graphql
```

Creates:

- `src/graphql/schema.rs`
- GraphQL bootstrap in `main.rs`
- GraphiQL mounted at `/`

Generate a gRPC-first app:

```bash
nestforge new demo-grpc --transport grpc
```

Creates:

- `proto/greeter.proto`
- `build.rs` for tonic code generation
- `src/grpc/service.rs`
- gRPC bootstrap in `main.rs`

Generate a WebSocket-first app:

```bash
nestforge new demo-events --transport websockets
```

Creates:

- `src/ws/events_gateway.rs`
- `src/ws/mod.rs`
- WebSocket bootstrap in `main.rs`

Generate a microservices-first app:

```bash
nestforge new demo-bus --transport microservices
```

Creates:

- `src/microservices/app_patterns.rs`
- `src/microservices/mod.rs`
- in-process client bootstrap in `main.rs`

### Module

```bash
nestforge g module users
```

Creates Nest-style feature structure:

- `src/users/mod.rs`
- `src/users/controllers/*`
- `src/users/services/*`
- `src/users/dto/*`

Also patches `main.rs` and `app_module.rs` imports.
For newer scaffolds, the CLI also patches the root `src/lib.rs` barrel so app-level imports can
stay flat.

### Resource In Module

```bash
nestforge g resource users --module users
```

Generates DTOs, service, controller inside the target module and wires exports/providers/controllers.

## Guard And Interceptor Generators

```bash
nestforge g guard auth
nestforge g decorator correlation_id
nestforge g filter rewrite_bad_request
nestforge g middleware audit
nestforge g interceptor logging
nestforge g serializer user
```

`nestforge g decorator <name>` creates `src/decorators/<name>_decorator.rs` plus export wiring in `src/decorators/mod.rs`.

`nestforge g filter <name>` creates `src/filters/<name>_filter.rs` plus export wiring in `src/filters/mod.rs`.

`nestforge g middleware <name>` creates `src/middleware/<name>_middleware.rs` plus export wiring in `src/middleware/mod.rs`.

`nestforge g serializer <name>` creates `src/serializers/<name>_serializer.rs` plus export wiring in `src/serializers/mod.rs`.

## GraphQL, gRPC, And Messaging Generators

Generate a GraphQL resolver stub:

```bash
nestforge g graphql users
```

Creates:

- `src/graphql/users_resolver.rs`
- export wiring in `src/graphql/mod.rs`

Generate a gRPC service stub:

```bash
nestforge g grpc billing
```

Creates:

- `proto/billing.proto`
- `src/grpc/billing_service.rs`
- `src/grpc/mod.rs` updates for the generated proto package and service export
- `build.rs` updates so tonic compiles the new proto file

Generate a WebSocket gateway stub:

```bash
nestforge g gateway events
```

Creates:

- `src/ws/events_gateway.rs`
- `src/ws/mod.rs` export wiring

Generate a microservice pattern registry stub:

```bash
nestforge g microservice users
```

Creates:

- `src/microservices/users_patterns.rs`
- `src/microservices/mod.rs` export wiring

The generated stub expects the `microservices` feature to be enabled on `nestforge`.

## DB Commands

### Init

```bash
nestforge db init
```

Creates:

- `migrations/`
- `.nestforge/applied_migrations.txt`

### Generate Migration

```bash
nestforge db generate create_users_table
```

Creates a timestamped SQL file in `migrations/`.

### Migrate

```bash
nestforge db migrate
```

Runs pending SQL migrations using `DATABASE_URL`.

### Status

```bash
nestforge db status
```

Shows `applied`, `pending`, and `drift` migration status.

## Utilities

- `nestforge docs`: opens the interactive CLI docs browser in your terminal.
  - Navigation:
    - `j` / `Down`: Next topic
    - `k` / `Up`: Previous topic
    - `PageDown`: Scroll content down
    - `PageUp`: Scroll content up
    - `/`: Search topics
    - `q` / `Esc`: Quit
  - Use `--no-tui` if you prefer plain text output for pipes or basic terminals.
  - Pass a topic like `nestforge docs modules` to jump directly to a page.
- `nestforge export-docs`: writes OpenAPI output for the current app
- `nestforge fmt`: runs `cargo fmt`

## Prelude and App Barrels

New scaffolds now generate:

- a root `src/lib.rs` barrel with `pub use` re-exports for top-level app symbols
- a slimmer `src/main.rs` that imports from the package crate
- `use nestforge::prelude::*;` for common framework imports

That gives generated apps a flatter import style, for example:

```rust
use demo_api::AppModule;
use nestforge::prelude::*;
```

## Notes

- Run generator commands inside an app folder (`Cargo.toml` + `src/`).
- Use `--module <feature>` to generate inside a feature module.
- GraphQL and gRPC app templates still include `APP_NAME` and an optional `DATABASE_URL` placeholder in `.env`.
- The microservices app template uses the `testing` feature as a lightweight runtime bootstrap for the in-process client example.
- The gRPC template expects `protoc` to be available when you build the generated app.
- The microservice generator expects the `microservices` feature to be enabled in the target app.
