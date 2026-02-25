# Example App Walkthrough

Path: `examples/hello-nestforge`

## Boot flow

1. `src/main.rs` calls `NestForgeFactory::<AppModule>::create()?.listen(3000)`
2. `src/app_module.rs` defines controllers + providers via `#[module]`
3. Factory builds container, registers providers, mounts controller routers

## Controllers

- `AppController`: `GET /` welcome message
- `HealthController`: `GET /health`
- `UsersController`: CRUD-like user routes

## Providers

- `AppConfig`: basic app config
- `UsersService`: user business logic backed by `InMemoryStore<UserDto>`

## DTOs

- `UserDto`: response model + `Identifiable`
- `CreateUserDto`: POST body + validation
- `UpdateUserDto`: PUT body + validation
