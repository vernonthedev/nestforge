# Cache

NestForge cache support is built around interceptors and the `CacheStore` abstraction.

Enable the `cache` feature and register a cache store provider such as `InMemoryRedisStore`.

## Cache Policy

Policies decide:

- which requests are cacheable
- how cache keys are built
- which store type is used
- optional TTL values

```rust
#[derive(Default, Clone)]
struct UsersCachePolicy;

impl nestforge::CachePolicy for UsersCachePolicy {
    type Store = nestforge::InMemoryRedisStore;
}
```

## Use The Interceptor

```rust
NestForgeFactory::<AppModule>::create()?
    .use_interceptor::<nestforge::CacheInterceptor<UsersCachePolicy>>();
```

By default, the cache interceptor:

- only caches `GET` requests
- keys entries by request URI
- only stores successful `200 OK` responses

Override those behaviors by implementing the `CachePolicy` methods.
