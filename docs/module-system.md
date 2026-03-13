# Module System

NestForge modules work like NestJS modules.

## Why Modules

Modules help you keep features isolated and readable:

- each feature has its own controllers/services/dtos
- dependencies are explicit
- startup wiring is predictable

## Module Definition

Use `#[module(...)]` on a struct.

Example:

```rust
use nestforge::prelude::*;
use crate::{UsersModule, SettingsModule, AppController, HealthController, Db};

#[module(
    imports = [UsersModule, SettingsModule],
    controllers = [AppController, HealthController],
    providers = [Provider::value(config), Provider::factory(build_service)],
    exports = [Db]
)]
pub struct AppModule;
```

## Providers and Dependency Injection

Providers are the core building blocks of a NestForge application. They can be injected into controllers, other providers, and guards.

### The `#[injectable]` Macro

Most services and providers should use the `#[injectable]` macro. This marks the struct as a managed provider and automatically implements `Clone`.

```rust
use nestforge::prelude::*;

#[injectable]
#[derive(Default)]
pub struct UsersService {
    // ...
}
```

By default, `#[injectable]` registers the provider using `Self::default()`.

#### Custom Factory

If your provider needs custom initialization, you can use the `factory` attribute:

```rust
use nestforge::prelude::*;

#[injectable(factory = build_auth_service)]
pub struct AuthService {
    secret: String,
}

fn build_auth_service() -> AuthService {
    AuthService {
        secret: "secure-secret".to_string(),
    }
}
```

#### Manual Registration

You can also register providers manually in the `#[module]` macro:

- `Provider::value(val)`: Use an existing value.
- `Provider::factory(|container| { ... })`: Use a closure to build the provider.
- `Provider::request_factory(|container| { ... })`: Create a new instance for every request.

## imports

`imports` means: "register these modules first".

NestForge traverses imports before registering current module providers.

## exports

`exports` means: "this module intentionally exposes these providers".

NestForge verifies every exported type is actually registered.

If not, startup fails with a clear error.

## Cycle Detection

If modules import each other in a loop, startup fails with a readable cycle path.

Example error style:

```text
Detected module import cycle: A -> B -> C -> A
```

## Resolution Order

For each module, NestForge does:

1. visit imports
2. register current module providers
3. mount current module controllers

This makes provider order deterministic.

## Lifecycle Hooks

Modules can also define Nest-style lifecycle hooks in `#[module(...)]`:

```rust
use nestforge::prelude::*;

#[module(
    providers = [load_config()?],
    on_module_init = [warm_cache],
    on_application_bootstrap = [log_startup],
    on_module_destroy = [flush_metrics],
    on_application_shutdown = [close_resources]
)]
pub struct AppModule;
```

Each hook is a function:

```rust
fn hook(container: &nestforge::Container) -> anyhow::Result<()>
```

Available hook lists:

- `on_module_init`
- `on_application_bootstrap`
- `on_module_destroy`
- `on_application_shutdown`

## Dynamic Modules

NestForge supports runtime-configured imports through `ModuleRef::builder(...)`.

That gives you a cleaner NestJS-style `register(...)` pattern:

```rust
use nestforge::prelude::*;

fn auth_module(secret: String) -> nestforge::ModuleRef {
    nestforge::ModuleRef::builder("AuthModule")
        .provider_value(AuthConfig { secret })
        .export::<AuthConfig>()
        .build()
}
```

You can also use factory-based registration when the module depends on other providers:

```rust
use nestforge::prelude::*;

fn auth_module() -> nestforge::ModuleRef {
    nestforge::ModuleRef::builder("AuthModule")
        .provider_factory(|container| {
            let config = container.resolve::<AppConfig>()?;
            Ok(AuthConfig {
                secret: config.jwt_secret.clone(),
            })
        })
        .export::<AuthConfig>()
        .build()
}
```

And async setup is available through `provider_async(...)`:

```rust
use nestforge::prelude::*;

fn remote_config_module() -> nestforge::ModuleRef {
    nestforge::ModuleRef::builder("RemoteConfigModule")
        .provider_async(|_container| async {
            Ok(RemoteConfig {
                region: "us-east-1".to_string(),
            })
        })
        .export::<RemoteConfig>()
        .build()
}
```

Then import it from a module:

```rust
use nestforge::prelude::*;

fn imports() -> Vec<nestforge::ModuleRef> {
    vec![auth_module("dev-secret".to_string())]
}
```

## Module Graph Introspection

Use `collect_module_graph::<AppModule>()` when you need a diagnostic snapshot of the resolved module tree.

Each reported module includes:

- module name
- imported modules
- exported provider types
- controller count
- whether the module is global
