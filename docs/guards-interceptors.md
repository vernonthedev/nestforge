# Guards And Interceptors

NestForge uses a pipeline model similar to NestJS.

## Guard

A guard decides if a request can continue.

- runs before handler logic
- returns `Ok(())` to continue
- returns `HttpException` to block

Quick macro style:

```rust
nestforge::guard!(RequireValidIdGuard, |ctx| {
    if let Some(last_segment) = ctx.uri.path().rsplit('/').next() {
        if last_segment == "0" {
            return Err(nestforge::HttpException::bad_request("id must be greater than 0"));
        }
    }
    Ok(())
});
```

Use per route:

```rust
#[nestforge::use_guard(crate::guards::RequireValidIdGuard)]
```

Use at controller level:

```rust
#[nestforge::routes]
#[nestforge::use_guard(crate::guards::RequireValidIdGuard)]
impl UsersController {
    // every route inherits the guard
}
```

Use globally:

```rust
NestForgeFactory::<AppModule>::create()?
    .use_guard::<AllowAllGuard>()
```

Auth helpers:

```rust
nestforge::auth_guard!(RequireAuthGuard);
nestforge::role_guard!(RequireAdminGuard, "admin");
```

Custom request decorators are available through `Decorated<T>` and `RequestDecorator`, which is useful for `@User()` or tenant-style extraction without coupling that logic to every controller method.

## Interceptor

An interceptor wraps handler execution.

Common use cases:

- request timing
- logging
- response shaping

Quick macro style:

```rust
nestforge::interceptor!(LoggingInterceptor, |ctx, req, next| {
    let started = std::time::Instant::now();
    let response = (next)(req).await;
    println!("{} {} - {}ms", ctx.method, ctx.uri, started.elapsed().as_millis());
    response
});
```

Use per route:

```rust
#[nestforge::use_interceptor(crate::interceptors::LoggingInterceptor)]
```

Use at controller level:

```rust
#[nestforge::routes]
#[nestforge::use_interceptor(crate::interceptors::LoggingInterceptor)]
impl UsersController {
    // every route inherits the interceptor
}
```

Use globally:

```rust
NestForgeFactory::<AppModule>::create()?
    .use_interceptor::<LoggingInterceptor>()
```

Cache interceptors are available through the optional `cache` feature:

```rust
#[derive(Default)]
struct UsersCache;

impl nestforge::CachePolicy for UsersCache {
    type Store = nestforge::InMemoryRedisStore;
}

NestForgeFactory::<AppModule>::create()?
    .use_interceptor::<nestforge::CacheInterceptor<UsersCache>>();
```

## Execution Order

Request flow:

1. global guards
2. route guards
3. interceptors (outer to inner)
4. handler

If any guard fails, handler is not called.

## Exception Filters

Global exception filters can rewrite `HttpException` values coming out of the framework pipeline:

```rust
#[derive(Default)]
struct RewriteBadRequestFilter;

impl nestforge::ExceptionFilter for RewriteBadRequestFilter {
    fn catch(
        &self,
        exception: nestforge::HttpException,
        _ctx: &nestforge::RequestContext,
    ) -> nestforge::HttpException {
        exception
    }
}

NestForgeFactory::<AppModule>::create()?
    .use_exception_filter::<RewriteBadRequestFilter>();
```

Use per route:

```rust
#[nestforge::use_exception_filter(crate::filters::RewriteBadRequestFilter)]
```

Use at controller level:

```rust
#[nestforge::routes]
#[nestforge::use_exception_filter(crate::filters::RewriteBadRequestFilter)]
impl UsersController {
    // every route inherits the exception filter
}
```
