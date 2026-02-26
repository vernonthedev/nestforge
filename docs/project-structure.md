# Project Structure

## Workspace Root

- `Cargo.toml`: workspace members and shared dependencies
- `README.md`: project overview
- `docs/`: markdown docs synced to wiki
- `crates/`: framework crates
- `examples/`: runnable example apps
- `tests/`: automation scripts

## Framework Crates

- `crates/nestforge`: public entry crate and re-exports
- `crates/nestforge-core`: DI container, module graph, routing helpers, validation, resource service
- `crates/nestforge-http`: app bootstrap (`NestForgeFactory`)
- `crates/nestforge-macros`: proc macros for modules/controllers/routes
- `crates/nestforge-cli`: CLI scaffolding and generators
- `crates/nestforge-config`: env/config loading and validation schema
- `crates/nestforge-db`: SQL-oriented DB wrapper and transaction APIs
- `crates/nestforge-orm`: ORM abstractions for relational data
- `crates/nestforge-data`: common patterns for non-relational adapters
- `crates/nestforge-openapi`: OpenAPI support surface
- `crates/nestforge-testing`: testing module factory and provider overrides

## Example App

`examples/hello-nestforge` shows current best-practice structure:

- root app files (`main.rs`, `app_module.rs`, `app_controller.rs`, `health_controller.rs`)
- feature modules (`users/`, `settings/`, `versioning/`)
- global guards/interceptors
- versioned routes and global `/api` prefix
