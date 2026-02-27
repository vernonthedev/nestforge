# Example App Walkthrough

Path: `examples/hello-nestforge`

## What This Example Shows

- Nest-style module imports (`UsersModule`, `SettingsModule`, `VersioningModule`)
- Root app controllers at `src/` level
- Feature folders for each domain
- Global guard + interceptor registration
- Route-level guards/interceptors
- Route versioning (`v1`, `v2`)
- Config loading with schema checks

## Boot Flow

1. `main.rs` creates the app with `NestForgeFactory::<AppModule>::create()`
2. Factory applies global prefix (`api`)
3. Factory registers global guard/interceptor
4. Module graph is resolved and providers/controllers are registered
5. HTTP server starts on port `3000`

## AppModule

`app_module.rs` does three main jobs:

- imports feature modules
- registers root controllers (`AppController`, `HealthController`)
- registers root providers (`AppConfig`, `Db`)

It also validates env values using:

- `ConfigModule::for_root`
- `ConfigOptions`
- `EnvSchema`

## Users Feature

`src/users/` contains:

- `controllers/users_controller.rs`
- `services/users_service.rs`
- `dto/*`

The controller demonstrates:

- full CRUD routes
- validation with `ValidatedBody<T>`
- route-level guards/interceptors
- cleaner error mapping with `or_bad_request()` and `or_not_found_id()`

## Settings Feature

`src/settings/` contains CRUD endpoints plus runtime config endpoint:

- `GET /api/v1/settings/runtime`

This endpoint shows DI config injection (`Inject<AppConfig>`).

## Versioning Feature

`src/versioning/controllers/versioning_controller.rs` has two versions of the same route:

- `GET /api/v1/versioning/hello`
- `GET /api/v2/versioning/hello`

## Guards And Interceptors

- Guards: `src/guards/`
- Interceptors: `src/interceptors/`

Example uses the macro-based short style (`nestforge::guard!`, `nestforge::interceptor!`) to keep code small.
