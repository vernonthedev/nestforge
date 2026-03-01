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
#[module(
    imports = [UsersModule, SettingsModule],
    controllers = [AppController, HealthController],
    providers = [Provider::value(config), Provider::factory(build_service)],
    exports = [Db]
)]
pub struct AppModule;
```

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
