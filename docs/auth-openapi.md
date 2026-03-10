# OpenAPI Setup from Scratch

NestForge provides built-in support for generating OpenAPI 3.1 specifications and serving them via a documentation UI. This guide covers how to set this up in a new project.

## 1. Enable the OpenAPI Feature

OpenAPI support is optional to keep the core framework lightweight. Enable it in your `Cargo.toml`:

```toml
[dependencies]
nestforge = { version = "1", features = ["openapi"] }
```

## 2. Activate Documentation Routes

In your `src/main.rs`, import `NestForgeFactoryOpenApiExt` and call `.with_openapi_docs()` during bootstrap.

```rust
use nestforge::{NestForgeFactory, NestForgeFactoryOpenApiExt};
use crate::app_module::AppModule;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    NestForgeFactory::<AppModule>::create()?
        .with_openapi_docs("My Service API", "1.0.0")?
        .listen(3000)
        .await?;
    Ok(())
}
```

This automatically mounts:

- `GET /openapi.json`: The raw OpenAPI spec.
- `GET /openapi.yaml`: The YAML export of the same spec.
- `GET /docs`: The primary docs UI.
- `GET /swagger-ui`: Swagger UI.
- `GET /redoc`: Redoc.

By default, NestForge now serves Swagger UI at `/docs` and also exposes dedicated `/swagger-ui` and `/redoc` routes.

If you are starting a new HTTP app, the CLI can scaffold this wiring for you:

```bash
nestforge new demo-api --openapi
```

## 2.1 Customize Docs Paths and UI

If you want docs mounted at `api/docs` or prefer Redoc as the default UI, use `OpenApiConfig`:

```rust
use nestforge::{
    NestForgeFactory, NestForgeFactoryOpenApiExt, OpenApiConfig, OpenApiUi,
};

NestForgeFactory::<AppModule>::create()?
    .with_openapi_docs_config(
        "My Service API",
        "1.0.0",
        OpenApiConfig::new()
            .with_docs_path("/api/docs")
            .with_default_ui(OpenApiUi::Redoc),
    )?
    .listen(3000)
    .await?;
```

## 3. Annotate Your Controllers

Use metadata attributes to describe your endpoints.

```rust
#[controller("/v1/items")]
pub struct ItemsController;

#[routes]
impl ItemsController {
    #[nestforge::get("/")]
    #[nestforge::summary("List all items")]
    #[nestforge::tag("Inventory")]
    #[nestforge::response(status = 200, description = "Success")]
    async fn list() -> ApiResult<Vec<ItemDto>> {
        // ...
    }
}
```

DTOs marked with `#[nestforge::dto]`, `#[nestforge::response_dto]`, or `#[nestforge::entity_dto]` are now emitted into `components.schemas` automatically. NestForge also infers request and response schemas from common controller signatures such as:

- `body: ValidatedBody<CreateUserDto>`
- `body: Body<CreateUserDto>`
- `-> ApiResult<UserDto>`
- `-> ApiResult<Vec<UserDto>>`

### Available Metadata Attributes

- `#[nestforge::summary("...")]`: Short summary of the route.
- `#[nestforge::description("...")]`: Detailed description.
- `#[nestforge::tag("...")]`: Group routes in the UI.
- `#[nestforge::response(status = 200, description = "...")]`: Document expected responses.
- `#[nestforge::authenticated]`: Marks the route as requiring authentication in the spec.

## 4. Authentication Integration

If you use `#[nestforge::authenticated]`, the generated spec will include the `bearerAuth` security scheme.

To resolve identities at runtime, configure an auth resolver:

```rust
NestForgeFactory::<AppModule>::create()?
    .with_auth_resolver(|token, _container| async move {
        // Map your token to an AuthIdentity here
        Ok(token.map(|t| nestforge::AuthIdentity::new("user_id")))
    })
    .with_openapi_docs("Secure API", "1.0.0")?
    .listen(3000)
    .await?;
```

## Troubleshooting

- **Missing Attributes**: Ensure `features = ["openapi"]` is in `Cargo.toml`.
- **Method Not Found**: Ensure you have `use nestforge::NestForgeFactoryOpenApiExt;` in scope.
- **Empty Docs**: Ensure your controllers are added to the `controllers` list in your module declaration.

## Static Export

NestForge CLI can export a static spec file from a standard app layout:

```bash
nestforge export-docs
nestforge export-docs --format yaml --output docs/openapi.yaml
```

The export command expects the app to depend on `nestforge` with the `openapi` feature enabled. It uses `app_module::AppModule` by default; override the type with `--module-type <TypeName>` if your root module uses a different name.
