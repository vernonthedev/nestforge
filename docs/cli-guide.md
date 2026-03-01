# CLI Guide

## Install

```bash
cargo install --path crates/nestforge-cli
```

## Command Overview

```text
nestforge new <app-name>
nestforge new <app-name> --transport <http|graphql|grpc|websockets>
nestforge g module <name>
nestforge g resource <name>
nestforge g controller <name>
nestforge g service <name>
nestforge g guard <name>
nestforge g interceptor <name>
nestforge g graphql <name>
nestforge g grpc <name>
nestforge db init
nestforge db generate <name>
nestforge db migrate
nestforge db status
nestforge docs
nestforge fmt
```

## Generators

### New App

```bash
nestforge new demo-api
```

Creates a runnable app with:

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

### Resource In Module

```bash
nestforge g resource users --module users
```

Generates DTOs, service, controller inside the target module and wires exports/providers/controllers.

## Guard And Interceptor Generators

```bash
nestforge g guard auth
nestforge g interceptor logging
```

## GraphQL And gRPC Generators

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

- `nestforge docs`: generates `docs/openapi.json` skeleton
- `nestforge fmt`: runs `cargo fmt`

## Notes

- Run generator commands inside an app folder (`Cargo.toml` + `src/`).
- Use `--module <feature>` to generate inside a feature module.
- GraphQL and gRPC app templates still include `APP_NAME` and an optional `DATABASE_URL` placeholder in `.env`.
- The gRPC template expects `protoc` to be available when you build the generated app.
