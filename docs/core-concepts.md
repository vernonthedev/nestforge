# Core Concepts

## Module

A module groups related controllers and providers.

With `#[module(...)]` you can define:

- `imports = [...]`
- `controllers = [...]`
- `providers = [...]`
- `exports = [...]`

NestForge registers imported modules first, then current module providers, then controllers.

## Module Graph

NestForge builds a module graph at startup.

It handles:

- import order
- shared module deduplication
- cycle detection with clear error messages

## Provider And DI

Providers are values in the DI container.

You can register providers with:

- direct values (`Provider::value(...)`)
- factories (`Provider::factory(|container| ...)`)

In handlers, use `Inject<T>` to resolve dependencies.

## Controllers

Controllers define HTTP routes.

- `#[controller("/users")]` sets a base path
- `#[routes]` maps methods into router entries
- `#[nestforge::get]`, `#[nestforge::post]`, `#[nestforge::put]`, `#[nestforge::delete]` define endpoints

## Request Types

- `Param<T>`: path params
- `Body<T>`: JSON body
- `ValidatedBody<T>`: JSON body + validation

## Validation

Use `ValidatedBody<T>` when `T` implements `Validate`.

Common pattern:

```rust
async fn create(body: ValidatedBody<CreateUserDto>) -> ApiResult<UserDto>
```

## Errors

Use `HttpException` for API errors.

Helper extensions make controllers simpler:

- `result.or_bad_request()?`
- `option.or_not_found_id("User", id)?`

## Guards And Interceptors

- `Guard`: authorization/route checks before handler
- `Interceptor`: wraps handler execution (logging, timing, mapping)

You can register globally in factory, and per-route via macros.

## Versioning And Prefix

- Global prefix: `.with_global_prefix("api")`
- Route version: `#[nestforge::version("1")]`

This yields routes like `/api/v1/users`.

## ResourceService

`ResourceService<T>` is a generic CRUD service on top of `InMemoryStore<T>`.

It gives simple methods like:

- `all()`
- `get(id)`
- `create(dto)`
- `update(id, dto)`
- `replace(id, dto)`
- `delete(id)`
