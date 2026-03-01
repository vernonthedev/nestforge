# Middleware

NestForge supports route-targeted HTTP middleware through `NestForgeFactory::configure_middleware(...)`.

## Middleware Type

`NestMiddleware` receives the raw request and a `next` callback.

```rust
use nestforge::{middleware, NextFn, NextFuture};

middleware!(RequestLogger, |req, next| {
    {
        println!("{} {}", req.method(), req.uri().path());
        (next)(req).await
    }
});
```

## Register Middleware

Use `configure_middleware(...)` to apply middleware globally or only for selected route prefixes.

```rust
use nestforge::{MiddlewareConsumer, NestForgeFactory};

let app = NestForgeFactory::<AppModule>::create()?
    .configure_middleware(|consumer: &mut MiddlewareConsumer| {
        consumer.apply::<RequestLogger>().for_all_routes();
        consumer
            .apply::<AdminAuditMiddleware>()
            .exclude(["/admin/health"])
            .for_routes(["/admin"]);
    });
```

`for_routes(["/users"])` matches `/users` and nested paths like `/users/1`.

You can also target specific HTTP methods:

```rust
use axum::http::Method;
use nestforge::MiddlewareRoute;

let app = NestForgeFactory::<AppModule>::create()?
    .configure_middleware(|consumer: &mut MiddlewareConsumer| {
        consumer
            .apply::<AdminAuditMiddleware>()
            .for_routes([MiddlewareRoute::get("/admin")]);

        consumer
            .apply::<WriteAuditMiddleware>()
            .for_routes([MiddlewareRoute::methods("/users", [Method::POST, Method::PUT])]);
    });
```

## Why This Exists

This fills the Nest-style gap between global app middleware and route-level guards/interceptors:

- middleware handles raw HTTP concerns early
- guards decide access
- interceptors shape execution around handlers

Use middleware for concerns like request logging, header normalization, tenant extraction, and legacy cookie/session adaptation.
