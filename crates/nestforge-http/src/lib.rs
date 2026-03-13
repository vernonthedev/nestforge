/**
 * NestForge HTTP Server Wrapper
 *
 * This crate provides the HTTP server infrastructure for NestForge applications.
 * It wraps axum and tokio to handle the complexity of web server setup,
 * allowing developers to focus on application logic.
 *
 * # Key Components
 * - `NestForgeFactory`: Main entry point for creating and configuring HTTP applications
 * - Middleware consumer and route configuration
 * - Request/response pipeline integration
 *
 * # Usage
 * Most users will interact with this crate through the main `nestforge` crate,
 * which re-exports the key types needed for application bootstrapping.
 */
pub mod factory;
pub mod middleware;

/**
 * Re-exports the application factory for creating NestForge HTTP servers.
 *
 * This is the primary way to create and configure a NestForge application
 * that serves HTTP endpoints.
 */
pub use factory::NestForgeFactory;

/**
 * Re-exports middleware types for request/response processing.
 *
 * Middleware in NestForge provides a way to execute logic before and after
 * request handling, similar to axum middleware but with NestForge-specific
 * integration.
 */
pub use middleware::{MiddlewareConsumer, MiddlewareRoute, NestMiddleware};
