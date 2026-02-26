# Contributing to NestForge

Thanks for contributing to NestForge.

This project is a Rust workspace for framework crates inspired by NestJS structure and built on Axum/Tokio. This guide explains how to propose changes, keep quality high, and get PRs merged quickly.

## Table of Contents

1. Code of Conduct
2. Ways to Contribute
3. Development Setup
4. Workspace Layout
5. Build, Test, and Lint
6. Coding Standards
7. Adding or Changing Features
8. Documentation Changes
9. Commit Message Rules
10. Pull Request Checklist
11. Release Notes and Versioning
12. Getting Help

## Code of Conduct

Be respectful, constructive, and focused on technical outcomes.

## Ways to Contribute

- Fix bugs
- Add framework features in the appropriate crate
- Improve docs and examples
- Add or improve tests
- Improve CLI ergonomics in `crates/nestforge-cli`

For large features or design shifts, open an issue first so we can align on scope.

## Development Setup

### Prerequisites

- Rust stable toolchain (Rust 2021 edition)
- `cargo`
- `git`

### Clone and bootstrap

```bash
git clone https://github.com/vernonthedev/nestforge.git
cd nestforge
cargo check
```

## Workspace Layout

Keep changes in the most specific crate.

- `crates/nestforge-core`: DI container, modules, route/request primitives
- `crates/nestforge-http`: HTTP app factory and runtime integration
- `crates/nestforge-macros`: procedural macros
- `crates/nestforge`: public framework crate (re-exports)
- `crates/nestforge-cli`: `nestforge` scaffolding binary
- `examples/hello-nestforge`: reference runnable app
- `docs/`: wiki-synced documentation content

Do not place framework internals in `examples/`.

## Build, Test, and Lint

Run from repository root.

### Fast compile validation

```bash
cargo check
```

### Build all crates and examples

```bash
cargo build --workspace
```

### Run reference app

```bash
cargo run -p hello-nestforge
```

### Run tests

```bash
cargo test --workspace
```

### Format code

```bash
cargo fmt --all
```

### Lint (CI-quality gate)

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Before opening a PR, at minimum run:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Coding Standards

- Rust 2021, 4-space indentation, `rustfmt` output
- Files/modules/functions: `snake_case`
- Types/traits/enums: `PascalCase`
- Constants/statics: `SCREAMING_SNAKE_CASE`
- Prefer small modules and explicit public APIs in each `lib.rs`

General guidance:

- Favor clear ownership/borrowing over unnecessary cloning
- Propagate errors with `Result` and descriptive error types
- Keep public APIs stable and intentional

## Adding or Changing Features

- Add behavior in the crate that owns it
- Keep changes scoped; avoid broad unrelated refactors
- Update tests with every behavior change
- Update docs when public APIs, CLI commands, or generated project structure changes

### Testing expectations

- Unit tests: inline with `#[cfg(test)]` in source files
- Integration tests: `crates/<crate>/tests/*.rs`
- Name tests by behavior, e.g. `resolves_provider_from_container`

## Documentation Changes

If behavior changes, update:

- `README.md` for user-facing usage changes
- `docs/` pages for framework concepts and guides
- Example app when it helps clarify usage

## Commit Message Rules

NestForge uses Conventional Commits and semantic release automation.

Use prefixes such as:

- `feat:`
- `fix:`
- `docs:`
- `refactor:`
- `chore:`

Scopes are encouraged:

- `feat(core): add request extractor`
- `fix(cli): support nested module generation`

Keep each commit focused and avoid mixing docs-only edits with functional changes when possible.

## Pull Request Checklist

Before requesting review, ensure:

- The PR has a clear summary and motivation
- Related issue is linked (if applicable)
- `cargo test --workspace` passes locally
- `cargo clippy --workspace --all-targets -- -D warnings` passes locally
- Docs were updated for public/API/CLI changes
- Changes are scoped and include tests for new behavior

### PR description template (recommended)

```md
## Summary

What changed and why.

## Changes

- Item 1
- Item 2

## Validation

- [x] cargo test --workspace
- [x] cargo clippy --workspace --all-targets -- -D warnings

## Docs

- [x] Updated docs/README (or N/A with reason)
```

## Release Notes and Versioning

Releases are automated from Conventional Commits on `main` via semantic-release. Write commit messages so release notes are accurate.

## Getting Help

- Open a GitHub issue for bugs/features
- Use discussions/issues for design questions
- For security-sensitive issues, avoid posting full exploit details publicly
