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
    if ctx.uri.path().ends_with("/0") {
        return Err(nestforge::HttpException::bad_request("id must be greater than 0"));
    }
    Ok(())
});
```

Use per route:

```rust
#[nestforge::use_guard(crate::guards::RequireValidIdGuard)]
```

Use globally:

```rust
NestForgeFactory::<AppModule>::create()?
    .use_guard::<AllowAllGuard>()
```

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

Use globally:

```rust
NestForgeFactory::<AppModule>::create()?
    .use_interceptor::<LoggingInterceptor>()
```

## Execution Order

Request flow:

1. global guards
2. route guards
3. interceptors (outer to inner)
4. handler

If any guard fails, handler is not called.
