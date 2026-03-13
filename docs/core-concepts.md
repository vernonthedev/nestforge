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

The recommended way to define providers is with the `#[injectable]` macro:

```rust
use nestforge::prelude::*;

#[injectable]
#[derive(Default)]
pub struct UsersService;
```

This marks the struct as a managed provider and automatically implements `Clone`. By default, it registers the provider using `Self::default()`.

For custom initialization, use the `factory` attribute:

```rust
#[injectable(factory = build_service)]
pub struct CustomService;
```

You can also register providers manually in a module with:

- direct values (`Provider::value(...)`)
- factories (`Provider::factory(|container| ...)`)
- request-scoped factories (`Provider::request_factory(|container| ...)`)
- transient factories (`Provider::transient_factory(|container| ...)`)

In handlers, use `Inject<T>` to resolve dependencies.

Request-scoped factories resolve against a per-request child container, so they can depend on request data like `RequestContext`, `RequestId`, or authenticated identity.

Transient factories build a fresh instance on every resolve and are useful for short-lived helper services.

## Controllers

Controllers define HTTP routes.

- `#[controller("/users")]` sets a base path
- `#[routes]` maps methods into router entries
- `#[nestforge::get]`, `#[nestforge::post]`, `#[nestforge::put]`, `#[nestforge::delete]` define endpoints

## Request Types

- `Param<T>`: path params
- `PipedParam<T, P>`: path params transformed by a pipe
- `Query<T>`: query params
- `PipedQuery<T, P>`: query params transformed by a pipe
- `Body<T>`: JSON body
- `PipedBody<T, P>`: JSON body transformed by a pipe
- `ValidatedBody<T>`: JSON body + validation
- `Decorated<T>`: custom request decorator extraction

Pipes let you transform or reject extracted values before handler logic runs:

```rust
struct SlugPipe;

impl nestforge::Pipe<String> for SlugPipe {
    type Output = String;

    fn transform(
        value: String,
        _ctx: &nestforge::RequestContext,
    ) -> Result<Self::Output, nestforge::HttpException> {
        Ok(value.trim().to_lowercase())
    }
}
```

Custom request decorators let you extract framework-specific values without hand-writing the same parsing code in every handler:

```rust
struct CorrelationId;

impl nestforge::RequestDecorator for CorrelationId {
    type Output = String;

    fn extract(
        _ctx: &nestforge::RequestContext,
        parts: &axum::http::request::Parts,
    ) -> Result<Self::Output, nestforge::HttpException> {
        parts
            .headers
            .get("x-correlation-id")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string)
            .ok_or_else(|| nestforge::HttpException::bad_request("Missing x-correlation-id"))
    }
}
```

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

## Response Envelopes

Use `ResponseEnvelope<T>` when you want a stable JSON shape across endpoints.

Common patterns:

```rust
async fn list() -> ApiEnvelopeResult<Vec<UserDto>> {
    Ok(ResponseEnvelope::paginated(users, 1, 20, 42))
}
```

This yields a payload shaped like:

- `success`
- `data`
- optional `meta`

## Response Serialization

Use `Serialized<T, S>` when a handler should return a domain type but expose a public DTO shape.

```rust
struct UserSerializer;

impl nestforge::ResponseSerializer<UserEntity> for UserSerializer {
    type Output = UserDto;

    fn serialize(value: UserEntity) -> Self::Output {
        UserDto {
            id: value.id,
            email: value.email,
        }
    }
}
```

Then return `ApiSerializedResult<UserEntity, UserSerializer>` from the handler.

The `hello-nestforge` example uses this pattern for `GET /api/info` by returning a serialized public view of `AppConfig`.

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
