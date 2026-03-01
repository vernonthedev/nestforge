# CLI Guide

## Install

```bash
cargo install --path crates/nestforge-cli
```

## Command Overview

```text
nestforge new <app-name>
nestforge new <app-name> --transport <http|graphql|grpc|microservices|websockets>
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

- `nestforge docs`: generates `docs/openapi.json` skeleton
- `nestforge fmt`: runs `cargo fmt`

## Notes

- Run generator commands inside an app folder (`Cargo.toml` + `src/`).
- Use `--module <feature>` to generate inside a feature module.
- GraphQL and gRPC app templates still include `APP_NAME` and an optional `DATABASE_URL` placeholder in `.env`.
- The microservices app template uses the `testing` feature as a lightweight runtime bootstrap for the in-process client example.
- The gRPC template expects `protoc` to be available when you build the generated app.
- The microservice generator expects the `microservices` feature to be enabled in the target app.
