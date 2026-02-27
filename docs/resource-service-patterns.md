# Resource Service Patterns

`ResourceService<T>` is a generic in-memory CRUD helper.

Use it to keep feature services small and readable in early app stages.

## Why It Helps

- avoids repetitive CRUD boilerplate
- keeps controller code clean
- still lets you add custom service functions when needed

## Basic Setup

```rust
pub type UsersService = nestforge::ResourceService<UserDto>;

pub fn users_service_seed() -> UsersService {
    UsersService::with_seed(vec![
        UserDto { id: 1, name: "Vernon".into(), email: "vernon@example.com".into() },
        UserDto { id: 2, name: "Sam".into(), email: "sam@example.com".into() },
    ])
}
```

## Methods

Main aliases available:

- `all()`
- `get(id)`
- `count()`
- `exists(id)`
- `create(dto)`
- `update(id, dto)`
- `replace(id, dto)`
- `delete(id)`

## DTO Requirements

For this service style:

- entity DTO should be serializable/deserializable
- entity DTO should implement `Identifiable`
- create/update DTOs should be serializable

NestForge macros help reduce boilerplate:

- `#[nestforge::dto]`
- `nestforge::impl_identifiable!(UserDto, id)`

## Controller Error Mapping

Pair `ResourceService` with helper methods:

- `.or_bad_request()?`
- `.or_not_found_id("User", id)?`

This keeps route handlers short and readable.
