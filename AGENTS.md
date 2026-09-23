I am a NestForge developer and you are my agent. We will be working together on NestForge, a Rust backend framework focused on bringing NestJS-style modularity and dependency injection to the Rust ecosystem. The goal is to keep NestForge fast, predictable, ergonomic, and easy to understand. Prefer simple designs that reduce framework complexity rather than introducing abstractions for their own sake. NestForge is a Rust workspace containing the core framework, HTTP runtime, procedural macros, CLI, configuration, database abstractions, optional transports, and example applications.

The primary workspace crates are:

* `crates/nestforge`: public crate users import
* `crates/nestforge-core`: DI, module graph, route builder, validation, and resource services
* `crates/nestforge-http`: application bootstrap and HTTP factory
* `crates/nestforge-macros`: framework procedural macros
* `crates/nestforge-cli`: `nestforge` CLI
* `crates/nestforge-config`: configuration loading and validation
* `crates/nestforge-db`: database wrapper and migration support
* `crates/nestforge-orm`: relational ORM abstractions
* `crates/nestforge-data`: non-relational data abstractions

Examples live under `examples/` and demonstrate real framework usage. Read the existing implementation and examples before introducing new framework patterns. The examples are part of the project's API design and should remain representative of how users are expected to build applications.

### Development Principles

* Prefer idiomatic, modern Rust.
* Keep APIs explicit, predictable, and type-safe.
* Prefer compile-time guarantees over runtime conventions where practical.
* Minimize unnecessary abstractions.
* Follow YAGNI.
* Do not introduce a dependency unless the need is concrete.
* Do not create an abstraction layer for a single implementation without a clear framework-level reason.
* Prefer the smallest implementation that solves the actual problem.
* If 200 lines can reasonably become 50 without sacrificing clarity, simplify it.
* Avoid clever Rust when straightforward Rust is easier to maintain.
* Keep public APIs ergonomic without hiding important behavior.
* Preserve backwards compatibility when modifying public APIs unless the change explicitly requires a breaking release.
* Touch only what is necessary for the requested change.
* Do not refactor unrelated code.
* Clean up only problems introduced by your own changes unless explicitly asked otherwise.
* Be careful with destructive changes.
* Do not delete existing user changes simply to make a branch cleaner.

### Framework Architecture

NestForge is intentionally modeled around concepts familiar to NestJS users, including:

* Modules
* Providers
* Dependency injection
* Controllers
* Routes
* Guards
* Interceptors
* Middleware
* Request extractors
* Configuration
* DTOs
* Validation
* Optional transports
* CLI generators

When changing one of these systems, first understand how the existing implementation connects across the relevant crates. Do not duplicate framework behavior in individual features when the behavior belongs in the framework core. Keep responsibilities separated between crates. For example:

* `nestforge-core` should contain framework-level dependency injection, module, routing, validation, and resource abstractions.
* `nestforge-http` should handle HTTP application bootstrap and HTTP-specific runtime behavior.
* `nestforge-macros` should contain procedural macro implementation.
* `nestforge-cli` should contain CLI and scaffolding behavior.
* Transport-specific functionality should remain isolated in its dedicated crate where possible.

Do not move functionality between crates simply for organizational preference. Make architectural changes only when there is a concrete dependency or responsibility problem.

### Feature-Based Organization

Prefer feature-oriented organization for application examples and generated projects.
NestForge's CLI supports flat feature layouts such as:

```text
src/users/
    mod.rs
    user_dto.rs
    create_user_dto.rs
    update_user_dto.rs
    users_controller.rs
    users_service.rs
```

Use the existing project conventions rather than introducing arbitrary folder structures. When modifying generators, ensure the generated project remains consistent with the framework's current conventions.

### Dependency Injection

Use NestForge's existing dependency injection mechanisms rather than creating parallel service registries or custom provider systems. Use `#[injectable]` for framework-managed providers where appropriate. Use explicit value or factory providers for external runtime resources such as database connections, clients, and other values that are not ordinary owned structs. Preserve the distinction between:

* framework-managed providers
* factory-created providers
* externally supplied runtime resources

Do not add special-case dependency injection behavior without understanding the existing module graph and provider resolution system.

### Macros

Procedural macros are part of NestForge's public developer experience.
When changing macros:

* Inspect the existing macro implementation before modifying it.
* Preserve generated code quality and diagnostics.
* Prefer clear compile-time errors over confusing runtime failures.
* Consider how generated code appears to framework users.
* Test both valid and invalid macro usage where practical.
* Keep generated APIs consistent with the handwritten APIs they represent.

Relevant framework macros include:

```rust
#[module]
#[controller]
#[routes]
#[get]
#[post]
#[put]
#[delete]
#[injectable]
```

Do not introduce a macro when a normal Rust API can solve the problem cleanly.

### Public API

Treat public types, traits, macros, builders, attributes, and re-exports as API contracts.
Before changing a public API:

* Search the workspace for all usages.
* Check the examples.
* Check generated code and CLI scaffolds where relevant.
* Consider downstream application code.
* Consider whether the change affects semver.
* Prefer additive changes when possible.

The `nestforge::prelude` is intentionally a lightweight collection of commonly used framework APIs. Keep it focused and avoid turning it into an indiscriminate export bucket.

### Rust Standards

* Do not use `unwrap()` or `expect()` in production framework code unless the invariant is genuinely guaranteed and the reason is clear.
* Prefer `Result` and `Option` for recoverable conditions.
* Use meaningful error types.
* Preserve error context when propagating failures.
* Avoid swallowing errors.
* Avoid unnecessary `clone()` calls.
* Avoid unnecessary allocations.
* Prefer borrowing when ownership does not need to change.
* Use `Arc`, `Mutex`, `RwLock`, channels, or other synchronization primitives only when their ownership or concurrency requirements justify them.
* Do not add `unsafe` without a strong reason and careful justification.
* Keep lifetimes as simple as possible.
* Prefer readable generic bounds over unnecessarily clever type-level designs.
* Avoid single-letter variable names except for conventional cases where the meaning is genuinely obvious.
* Use descriptive names for types, functions, variables, modules, and generic parameters.
* Keep comments focused on why something exists rather than restating what the code already says.

### Async and Runtime Behavior

NestForge is an asynchronous backend framework.
When modifying async code:

* Avoid blocking the async runtime.
* Do not introduce synchronous I/O into async paths without understanding the consequences.
* Consider cancellation and failure propagation.
* Avoid unnecessary task spawning.
* Do not spawn background tasks merely to avoid dealing with ownership or lifetimes.
* Preserve predictable shutdown behavior.
* Consider concurrency and shared-state behavior before changing runtime code.

Performance matters, but correctness and predictable behavior come first.
Do not optimize based on assumptions. Measure or identify a concrete bottleneck before introducing complexity specifically for performance.

### CLI and Generators

The `nestforge` CLI is part of the developer experience and should be treated as a first-class component of the project.
When changing generators:

* Inspect the generated output.
* Keep generated projects compilable.
* Preserve existing generator conventions.
* Keep interactive and non-interactive behavior consistent.
* Ensure generated imports and module declarations are correct.
* Update examples or generator tests when the generated structure changes.

Current CLI capabilities include application scaffolding, modules, resources, guards, filters, middleware, interceptors, GraphQL, gRPC, gateways, database commands, documentation generation, and formatting. Do not change generated project structure casually. Generator output is effectively part of NestForge's public API.

### Optional Features and Transports

NestForge supports optional functionality including:

* OpenAPI
* GraphQL
* gRPC
* WebSockets
* Scheduler
* Database integrations
* ORM abstractions
* Non-relational data abstractions

Keep optional functionality genuinely optional. Do not force an optional transport or dependency into the core framework when it can remain isolated behind its existing crate or feature boundary. When modifying an optional integration, verify that the default framework remains unaffected when that feature is disabled.

### Examples Are Contracts

The example applications are not disposable demos.
They demonstrate how users are expected to use NestForge and should compile against the current public API.
Relevant examples include:

```text
examples/hello-nestforge
examples/hello-nestforge-graphql
examples/hello-nestforge-grpc
examples/hello-nestforge-microservices
examples/hello-nestforge-websockets
```

When changing public framework behavior, inspect the affected examples. If an API change makes an example awkward or unnecessarily verbose, consider whether the framework API itself should be improved before modifying the example around it.

### Testing

Before writing or changing tests, inspect the existing testing patterns in the repository.
Tests should verify behavior rather than implementation details.
For framework features, prefer testing the public API that users actually interact with.
When appropriate, test:

* successful behavior
* invalid input
* error propagation
* provider resolution
* module boundaries
* route registration
* macro expansion behavior
* generated code
* CLI output
* feature-gated behavior

Do not add superficial tests purely to increase coverage.
Run the narrowest relevant tests first, then broader workspace checks when appropriate.
Useful repository checks include:

```bash
cargo check --workspace
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features
```

Use the repository's existing CI and release configuration as the source of truth if these commands differ from the current project setup.

### Documentation

Documentation is part of the public API.
Before changing documentation, inspect the existing documentation structure and conventions.
Keep documentation:

* accurate
* concise
* example-driven
* consistent with the current implementation

Do not create unnecessary Markdown files.
Do not document behavior that the implementation does not actually support.
When changing a public API, update the relevant documentation and examples when required.
Avoid using `---` as a section separator in Markdown documentation.
Never use em dashes in project documentation.

### Commit Instructions

All commits must follow Conventional Commits.
Use the project's existing commit history as the primary reference for scopes and wording.
Supported commit types include:

```text
feat
fix
refactor
docs
style
test
chore
ci
perf
```

Use a relevant scope whenever one can be identified.
Use one conceptual change per commit.
Commit messages should explain why the change matters rather than simply restating the files changed.

Preferred format:

```text
<type>(<scope>): <short summary>

- Explain the reason for the change.
- Explain the important behavior or outcome.
```

For example:

```text
fix(di): preserve provider resolution across imported modules

- Keep exported providers available to modules importing the provider module.
- Prevent provider lookup from incorrectly stopping at the local module boundary.
```

Do not add emojis to commit messages. Do not create empty commits. If the user explicitly requests a specific number of commits, create exactly that number of meaningful commits. Each commit must contain an actual conceptual change. Do not manufacture empty changes simply to satisfy the count. Never discard existing changes when asked to commit. Commit the relevant changes from the current working tree.

### Pull Requests

- Always use `.patterns/docs.md` when your creating pull requests, always first read this file before creating any pull request.
PR titles should use Conventional Commit format. Prefer concise, human-readable titles that explain why the change matters.

Example:

```text
fix(cli): keep generated feature modules importable
```

The PR description should begin with a plain explanation of the problem and the resulting solution.
Do not begin with:

```text
#### Summary
```

or:

```text
#### Description
```

Do not repeat the PR title as a heading.
Do not include:

* checkboxes
* TODO lists
* task lists
* a `Test plan` section
* a `Testing` section

The PR description should document the change rather than provide a verification checklist.
Use additional `####` headings only when the change genuinely requires multiple sections.

### Git Hygiene

Before committing or opening a PR:

* Inspect the current branch.
* Inspect the working tree.
* Review the final diff.
* Confirm unrelated changes have not been included.
* Check that the change is still relevant and has not been superseded.
* Keep the final diff minimal and reviewable.

Never reset, revert, delete, or overwrite existing user changes unless explicitly requested.
Do not clean up unrelated pre-existing code simply because you encountered it while working.

### Release System

NestForge uses a Rust-native release flow driven by the repository release script.
Conventional Commits determine semantic version bumps.
Changed crates are versioned, tagged, released on GitHub, and published to crates.io in dependency order.
The primary published changelog is:

```text
crates/nestforge/CHANGELOG.md
```

When making release-related changes, inspect the existing release scripts and workflow before modifying them. Do not manually invent version numbers when the repository's release tooling determines them. First-time publishing of a new crate may require the documented manual bootstrap process.

### Repository Rules

* Use Cargo for Rust dependency management and builds.
* Do not introduce another package manager for Rust tooling.
* Do not add dependencies without a concrete requirement.
* Do not hand-edit generated files when a generator exists.
* Do not modify generated output as a substitute for fixing the generator.
* Do not edit example output simply to hide framework regressions.
* Do not create unnecessary Markdown documentation.
* Never use emojis in source code, documentation, commits, console output, or generated project files.
* Never use em dashes in source documentation or project-facing text.
* Do not create custom components, abstractions, or framework APIs without first checking whether an existing NestForge primitive already solves the problem.
* Do not make destructive changes without explicit instruction.
* Do not make broad refactors while implementing a focused feature or fix.

### Working With the Agent

Use the repository as the source of truth.

Before making assumptions:

1. Inspect the relevant crate.
2. Search for existing implementations and usages.
3. Inspect affected examples.
4. Check tests and macros when applicable.
5. Make the smallest change that solves the problem.

Ask for clarification when the requested behavior is genuinely ambiguous and cannot be resolved from the repository. Do not ask questions that can be answered by inspecting the codebase. When proposing a new framework feature, explain the architectural reason for introducing it and how it fits into the existing NestForge model.
Prefer a focused implementation over broad parallel exploration. Use parallel work only when the task is genuinely decomposable into independent workstreams.

### Maintaining This File

This file should contain stable project facts and development rules. Do not turn it into a general software engineering essay.
If a rule only applies to a specific crate, feature, example, release workflow, or integration, put that guidance in the most relevant project-specific documentation instead. Keep this file accurate. If the repository architecture changes, update or remove rules that no longer apply. A stale `AGENTS.md` is worse than having no rule for the affected behavior.
