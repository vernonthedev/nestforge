# Full File Map

This map lists the major files and what they do.

## Root

- `Cargo.toml`: workspace configuration
- `README.md`: project intro and feature summary
- `CONTRIBUTING.md`: contribution process
- `LICENSE`: Apache-2.0 license text
- `docs/`: user and contributor docs
- `tests/`: test automation scripts

## crates/nestforge

- `src/lib.rs`: public re-exports, helper macros (`guard!`, `interceptor!`, `impl_identifiable!`)

## crates/nestforge-core

- `src/container.rs`: DI container
- `src/module.rs`: module graph traversal, imports/exports checks, cycle detection
- `src/provider.rs`: provider API (`Provider::value`, `Provider::factory`)
- `src/request.rs`: `Param<T>`, `Body<T>`, `ValidatedBody<T>`
- `src/pipeline.rs`: guard/interceptor pipeline execution
- `src/route_builder.rs`: route registration + version path support
- `src/resource_service.rs`: generic in-memory CRUD service
- `src/http_ext.rs`: `or_bad_request`, `or_not_found_id` helpers
- `src/error.rs`: `HttpException`
- `src/validation.rs`: validation traits/types

## crates/nestforge-http

- `src/factory.rs`: `NestForgeFactory` bootstrap, global prefix, global guards/interceptors, server listen

## crates/nestforge-macros

- `src/lib.rs`: module/controller/routes proc macros and route metadata macros

## crates/nestforge-config

- `src/lib.rs`: env loading, typed config parsing, schema validation (`EnvSchema`)

## crates/nestforge-db

- DB config, DB wrapper, transaction API, migration helpers used by CLI

## crates/nestforge-orm

- repository abstractions over DB layer

## crates/nestforge-data

- shared abstractions for non-relational backends

## crates/nestforge-cli

- `src/main.rs`: app scaffolding, generators, DB commands, docs and format commands

## examples/hello-nestforge

### Root files

- `src/main.rs`: startup and global middleware setup
- `src/app_module.rs`: root module wiring
- `src/app_config.rs`: typed config model (`FromEnv`)
- `src/app_controller.rs`: root endpoint
- `src/health_controller.rs`: health endpoints

### Cross-cutting

- `src/guards/*`: guard examples
- `src/interceptors/*`: interceptor examples

### Feature modules

- `src/users/*`: users DTOs, service, controller
- `src/settings/*`: settings DTOs, service, controller
- `src/versioning/*`: versioning demo controller
