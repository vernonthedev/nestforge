# CLI Guide

## Install

```bash
cargo install --path crates/nestforge-cli
```

## Command Overview

```text
nestforge new <app-name>
nestforge g module <name>
nestforge g resource <name>
nestforge g controller <name>
nestforge g service <name>
nestforge g guard <name>
nestforge g interceptor <name>
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
