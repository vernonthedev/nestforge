# Testing

NestForge includes a testing module builder through the optional `testing` feature.

## Enable The Feature

```toml
nestforge = { version = "1", features = ["testing"] }
```

## Build A Testing Module

```rust
let module = nestforge::TestFactory::<AppModule>::create()
    .override_provider(AppConfig {
        app_name: "test",
    })
    .build()?;
```

The built `TestingModule` can still resolve providers directly:

```rust
let config = module.resolve::<AppConfig>()?;
```

When a test finishes with modules that own cleanup work, call:

```rust
module.shutdown()?;
```

That runs module destroy and application shutdown lifecycle hooks inside the testing runtime.

## Build An HTTP Router

`TestingModule::http_router()` merges the module controllers into a ready-to-test Axum router with the framework container attached as state.

```rust
let app = module.http_router();
```

This is useful for request-level tests with `tower::ServiceExt::oneshot(...)`.

## Build A GraphQL Router

`TestingModule::graphql_router(...)` and `TestingModule::graphql_router_with_paths(...)` mount a schema into the same testing container:

```rust
let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
let app = module.graphql_router(schema);
```

That lets GraphQL resolvers resolve the same providers and overrides as the rest of the test module.

## Build Transport Contexts

`TestingModule` can also create transport contexts directly for lower-level tests:

```rust
let grpc = module.grpc_context();
let websocket = module.websocket_context();
let microservice = module.microservice_context("test", "users.count");
```

Use these helpers when you want to test gRPC services, websocket gateways, or transport-neutral message handlers without booting the full transport server.
