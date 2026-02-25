# Project Structure

## Root

- `Cargo.toml`: workspace config
- `README.md`: project intro
- `docs/`: onboarding docs
- `crates/`: framework crates
- `examples/`: sample apps

## Crates

- `crates/nestforge`: public crate users import
- `crates/nestforge-core`: DI, traits, request helpers, errors, store
- `crates/nestforge-http`: app factory + server startup
- `crates/nestforge-macros`: `#[module]`, `#[controller]`, `#[routes]`, HTTP method attrs
- `crates/nestforge-cli`: scaffolding and code generation commands

## Example

- `examples/hello-nestforge`: working reference app
