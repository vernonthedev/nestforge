# Macros

## `#[module(...)]`

Adds `ModuleDefinition` implementation for a struct.

Inputs:

- `controllers = [ControllerA, ControllerB]`
- `providers = [ServiceA::new(), AppConfig { ... }]`

Output:

- registers providers in the container
- returns controller routers

## `#[controller("/base")]`

Adds base-path metadata for a controller struct.

## `#[routes]`

Reads method attributes inside `impl` and builds router mappings.

Supported method attrs:

- `#[get("/path")]`
- `#[post("/path")]`
- `#[put("/path")]`

Also supports namespaced style:

- `#[nestforge::get("/path")]`
- `#[nestforge::post("/path")]`
- `#[nestforge::put("/path")]`

## What gets generated

`#[routes]` implements `ControllerDefinition::router()` and uses `RouteBuilder` under the hood.
