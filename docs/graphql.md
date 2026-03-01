# GraphQL

NestForge can expose GraphQL endpoints through the optional `nestforge-graphql` crate.

## Enable The Feature

```toml
nestforge = { version = "1", features = ["graphql"] }
```

## Minimal Setup

```rust
use nestforge::{
    async_graphql::{EmptyMutation, EmptySubscription, Object, Schema},
    NestForgeFactory, NestForgeFactoryGraphQlExt,
};

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn health(&self) -> &str {
        "ok"
    }
}

let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

NestForgeFactory::<AppModule>::create()?
    .with_graphql(schema)
    .listen(3000)
    .await?;
```

Default routes:

- `/graphql`
- `/graphiql`

## Custom Routes

```rust
use nestforge::{graphql_router_with_config, GraphQlConfig};

let router = graphql_router_with_config(
    schema,
    GraphQlConfig::new("/api/graphql").with_graphiql("/api/graphiql"),
);
```

That router can also be mounted manually with `NestForgeFactory::merge_router(...)`.
