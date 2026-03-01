# Auth And OpenAPI

NestForge now includes basic authentication primitives and generated OpenAPI support.

## Auth

Available extractors:

- `AuthUser`: requires an authenticated identity to already exist on the request
- `OptionalAuthUser`: resolves to `None` when the request is anonymous
- `BearerToken`: extracts a `Bearer ...` token from the `Authorization` header

Available guard macros:

```rust
nestforge::auth_guard!(RequireAuthGuard);
nestforge::role_guard!(RequireAdminGuard, "admin");
```

Runtime auth is configured at the factory level:

```rust
NestForgeFactory::<AppModule>::create()?
    .with_auth_resolver(|token, _container| async move {
        Ok(token.map(|_| nestforge::AuthIdentity::new("demo-user").with_roles(["admin"])))
    })
```

## OpenAPI

Controller methods can now declare metadata:

```rust
#[nestforge::get("/users")]
#[nestforge::authenticated]
#[nestforge::roles("admin", "support")]
#[nestforge::summary("List users")]
#[nestforge::tag("users")]
#[nestforge::response(status = 200, description = "Users returned")]
async fn list() -> nestforge::ApiResult<Vec<UserDto>> {
    # todo!()
}
```

`#[authenticated]` now enforces runtime authentication, and `#[roles(...)]` enforces that the authenticated identity has at least one required role.

Generated docs can be mounted directly:

```rust
use nestforge::{NestForgeFactory, NestForgeFactoryOpenApiExt};

NestForgeFactory::<AppModule>::create()?
    .with_openapi_docs("My API", "1.0.0")?
    .listen(3000)
    .await?;
```

This mounts:

- `/openapi.json`
- `/docs`
