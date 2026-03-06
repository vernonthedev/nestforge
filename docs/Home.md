# Welcome to NestForge

NestForge is a modular backend framework for Rust, heavily inspired by the architecture of NestJS. It provides a structured way to build scalable, maintainable, and high-performance server-side applications.

## Getting Started

If you are new to NestForge, we recommend following these guides in order:

1. **[Quick Start](./quick-start.md)**: Create and run your first app in 5 minutes.
2. **[Installation Guide](./project-structure.md)**: Detail on workspace layout and prerequisites.
3. **[Core Concepts](./core-concepts.md)**: Understand the mental model behind NestForge.
4. **[Module System](./module-system.md)**: How dependency injection and wiring works.
5. **[CLI Guide](./cli-guide.md)**: Speed up development with generators.

## Guided Workflows

Step-by-step processes for common tasks:

- **[OpenAPI Documentation Setup](./auth-openapi.md)**: Set up auto-generated docs from scratch.
- **[Relational Databases](./resource-service-patterns.md)**: CRUD patterns and service layers.
- **[Testing Workflow](./testing.md)**: How to write unit and integration tests.
- **[Configuration](./config-module.md)**: Managing environment variables and secrets.

## Advanced Transports

NestForge supports multiple protocols out of the box:

- **[GraphQL](./graphql.md)**: Using Apollo-style resolvers.
- **[gRPC](./grpc.md)**: Professional microservice communication.
- **[WebSockets](./websockets.md)**: Real-time bi-directional events.
- **[Microservices](./microservices.md)**: In-process and distributed message buses.

---

## Why NestForge?

- **Modular**: Encourages grouping features into logical bundles.
- **Type-Safe**: Leverages Rust's powerful type system for DI and validation.
- **Productive**: CLI and macros remove the boilerplate typically associated with Rust web-dev.
- **Flexible**: Choose your transport: REST, GraphQL, gRPC, or even raw WebSockets.
