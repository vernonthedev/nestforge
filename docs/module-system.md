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
