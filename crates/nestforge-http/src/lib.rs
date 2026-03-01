/*!
This crate wraps the actual HTTP server setup (axum + tokio stuff).

Goal:
Hide Rust web setup complexity from the NestForge user.
*/

pub mod factory;
pub mod middleware;

/*
Re-export the app factory so the public crate can expose it.
*/
pub use factory::NestForgeFactory;
pub use middleware::{MiddlewareConsumer, NestMiddleware};
