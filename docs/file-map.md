# Full File Map

This map documents every current source file and what it does.

## Workspace root

- `Cargo.toml`: workspace members + shared dependency versions
- `Cargo.lock`: resolved dependency lockfile
- `README.md`: public project intro
- `.gitignore`: ignored files

## `crates/nestforge` (public entry crate)

- `crates/nestforge/Cargo.toml`: package + internal crate deps
- `crates/nestforge/src/lib.rs`: public re-exports for users

## `crates/nestforge-core`

- `crates/nestforge-core/Cargo.toml`: core crate deps
- `crates/nestforge-core/src/lib.rs`: core module exports
- `crates/nestforge-core/src/module.rs`: module/controller trait contracts
- `crates/nestforge-core/src/container.rs`: DI container implementation
- `crates/nestforge-core/src/inject.rs`: `Inject<T>` request extractor
- `crates/nestforge-core/src/request.rs`: `Param<T>` and `Body<T>` wrappers
- `crates/nestforge-core/src/route_builder.rs`: route-builder utility for generated routers
- `crates/nestforge-core/src/error.rs`: `HttpException` + JSON error response
- `crates/nestforge-core/src/store.rs`: `Identifiable` + `InMemoryStore<T>`

## `crates/nestforge-http`

- `crates/nestforge-http/Cargo.toml`: HTTP crate deps
- `crates/nestforge-http/src/lib.rs`: exports `NestForgeFactory`
- `crates/nestforge-http/src/factory.rs`: bootstraps app, merges routers, starts server

## `crates/nestforge-macros`

- `crates/nestforge-macros/Cargo.toml`: proc-macro crate deps
- `crates/nestforge-macros/src/lib.rs`: macro implementations and parser helpers

## `crates/nestforge-cli`

- `crates/nestforge-cli/Cargo.toml`: CLI crate deps
- `crates/nestforge-cli/src/main.rs`: CLI commands, templates, file patchers, naming helpers

## `examples/hello-nestforge`

- `examples/hello-nestforge/Cargo.toml`: sample app deps
- `examples/hello-nestforge/src/main.rs`: app entrypoint
- `examples/hello-nestforge/src/app_module.rs`: app module configuration

### Controllers

- `examples/hello-nestforge/src/controllers/mod.rs`: controller exports
- `examples/hello-nestforge/src/controllers/app_controller.rs`: root endpoint
- `examples/hello-nestforge/src/controllers/health_controller.rs`: health endpoint
- `examples/hello-nestforge/src/controllers/users_controller.rs`: users endpoints

### Services

- `examples/hello-nestforge/src/services/mod.rs`: service exports
- `examples/hello-nestforge/src/services/app_config.rs`: app config provider
- `examples/hello-nestforge/src/services/users_service.rs`: user logic + store usage

### DTOs

- `examples/hello-nestforge/src/dto/mod.rs`: DTO exports
- `examples/hello-nestforge/src/dto/user_dto.rs`: user response/entity struct
- `examples/hello-nestforge/src/dto/create_user_dto.rs`: create request DTO + validation
- `examples/hello-nestforge/src/dto/update_user_dto.rs`: update request DTO + validation
