# NestForge Framework + ORM Roadmap

This document tracks the next major implementation phases for NestForge.

## Phase 0 - Module Graph Foundation

Status: In progress (core graph support implemented)

- [x] Add `imports()` support in `ModuleDefinition`
- [x] Add `exports()` support in `ModuleDefinition` (metadata)
- [x] Add `ModuleRef` for typed module references
- [x] Implement DFS module graph registration:
  - [x] register imports first
  - [x] register local module providers
  - [x] aggregate controllers
- [x] Add cycle detection with friendly error text
- [x] Add tests for:
  - [x] import registration order
  - [x] shared import dedupe
  - [x] cycle detection
- [ ] Add exported-provider visibility enforcement

## Phase 1 - Framework Production Readiness

### A1 Module Enhancements

- [x] Macro support for `imports = [...]`
- [x] Macro support for `exports = [...]`
- [ ] Global modules support
- [ ] Cross-module export visibility checks

### A2 Provider Lifecycle and DI

- [x] Introduce provider registry API
  - [x] `Provider::value(expr)`
  - [x] `Provider::factory(|container| ...)`
- [x] Support mixed provider syntax in `#[module(providers = [...])]`:
  - [x] plain values (auto-wrapped as `Provider::value`)
  - [x] explicit provider helpers (`Provider::value`, `Provider::factory`)
- [x] Typed DI errors with:
  - [x] requested type
  - [x] current module (during module registration via contextual resolve + graph error wrapping)
- [ ] Prepare lifecycle model:
  - [ ] singleton (default)
  - [ ] transient (planned)
  - [ ] request-scoped (planned)

### A3 Guards / Interceptors

- [ ] Add `Guard` trait
- [ ] Add `Interceptor` trait
- [ ] Add request context type
- [ ] Add route metadata + execution hooks
- [ ] Add example auth guard + logging interceptor

### A4 Validation / Pipes

- [x] Add `Validate` trait
- [x] Add `ValidatedBody<T>` extractor that auto-runs validation
- [x] Return structured 400 payloads with validation details
- [ ] Optional future: make `Body<T>` invoke validation automatically (requires specialization-like approach)

### A5 Config Module

- [ ] Add config crate/module (`dotenv` + env parsing)
- [ ] Add typed config extraction (`AppConfig`)
- [ ] Register config in DI (`Inject<AppConfig>`)
- [ ] CLI: generate `.env` + `.env.example`

### A6 Testing Utilities

- [x] Create `nestforge-testing` crate
- [x] Add `TestFactory::create::<M>()`
- [x] Add `.override_provider::<T>(mock)` (value override)
- [ ] Add override semantics for provider dependencies instantiated during module boot

### A7 OpenAPI

- [ ] Create `nestforge-openapi` crate
- [ ] Generate minimal OpenAPI document
- [ ] Serve `/openapi.json` and `/docs`
- [ ] CLI: `nestforge docs`

## Phase 2 - Tier 1 Data Layer (`nestforge-db`)

- [ ] Add `crates/nestforge-db`
- [ ] Implement `DbConfig`
- [ ] Implement `DbError` (`thiserror`)
- [ ] Implement injectable `Db`
- [ ] Implement transactions API
- [ ] Support named connections
- [ ] Integrate with module system (`DbModule`)
- [ ] Add example using `Inject<Db>`

## Phase 3 - Tier 2 Relational ORM (`nestforge-orm`)

- [ ] Add `crates/nestforge-orm`
- [ ] Add `#[entity(table = \"...\")]`
- [ ] Add `#[id]` support
- [ ] Add `Repo<T>` CRUD:
  - [ ] `find_all`
  - [ ] `find_by_id`
  - [ ] `create`
  - [ ] `update_by_id`
  - [ ] `delete_by_id`
- [ ] Add migration format and execution support
- [ ] CLI commands:
  - [ ] `nestforge db init`
  - [ ] `nestforge db generate`
  - [ ] `nestforge db migrate`
  - [ ] `nestforge db status`

## Phase 4 - Tier 3 Multi-Backend Data (`nestforge-data`)

- [ ] Add `crates/nestforge-data` abstractions
- [ ] Add `DocumentRepo<T>`
- [ ] Add `CacheStore`
- [ ] Add `crates/nestforge-mongo` adapter
- [ ] Add `crates/nestforge-redis` adapter
- [ ] Standardize error mapping

## CLI Evolution

- [ ] Ensure installable command:
  - [ ] `cargo install --path crates/nestforge-cli`
  - [ ] binary name `nestforge`
- [ ] Add command: `nestforge g module <name>`
- [ ] Add command group: `nestforge db *`
- [ ] Update templates to current API:
  - [ ] `NestForgeFactory::<AppModule>::create()?.listen(3000).await`
  - [ ] `#[module(imports, controllers, providers, exports)]`
- [ ] Add optional command: `nestforge fmt`

## Validation Checklist for Every Phase

- [ ] `cargo check`
- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
