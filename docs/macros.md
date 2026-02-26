# Macros

## Controller Macros

### `#[controller("/base")]`

Sets a base path for the controller.

### `#[routes]`

Builds Axum routes from methods in the `impl` block.

### HTTP Method Attributes

Supported route attributes:

- `#[nestforge::get("/path")]`
- `#[nestforge::post("/path")]`
- `#[nestforge::put("/path")]`
- `#[nestforge::delete("/path")]`

## Route Metadata Macros

### `#[nestforge::version("1")]`

Adds route version prefix (for example `/v1`).

### `#[nestforge::use_guard(MyGuard)]`

Adds a route-level guard.

### `#[nestforge::use_interceptor(MyInterceptor)]`

Adds a route-level interceptor.

## Module Macro

### `#[module(...)]`

Defines module wiring and generates `ModuleDefinition`.

Supported keys:

- `imports = [UsersModule, AuthModule]`
- `controllers = [UsersController]`
- `providers = [Provider::value(...), Provider::factory(...)]`
- `exports = [UsersService]`

## DTO Convenience Macros

NestForge also provides DTO helpers:

- `#[nestforge::dto]`
- `nestforge::impl_identifiable!(Type, id_field)`

These keep DTO boilerplate small while remaining explicit.

## Guard And Interceptor Short Macros

### `nestforge::guard!(Name)`

Creates a default allow guard quickly.

### `nestforge::guard!(Name, |ctx| { ... })`

Creates a custom guard with inline logic.

### `nestforge::interceptor!(Name)`

Creates a pass-through interceptor.

### `nestforge::interceptor!(Name, |ctx, req, next| { ... })`

Creates a custom interceptor inline.
