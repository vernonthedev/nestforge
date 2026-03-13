use axum::{
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use serde::Serialize;
use serde_json::json;

use crate::HttpException;

/**
 * ResponseSerializer Trait
 *
 * A trait for defining custom response serialization logic.
 * Use this when you want to transform a domain object into a specific
 * API response format, such as hiding fields, renaming keys, or
 * flattening structures.
 *
 * # Type Parameters
 * - `T`: The input type to serialize
 *
 * # Associated Types
 * - `Output`: The serialized output type (must implement Serialize)
 *
 * # Example
 * ```rust
 * struct UserResponseSerializer;
 * impl ResponseSerializer<User> for UserResponseSerializer {
 *     type Output = UserResponse;
 *     fn serialize(value: User) -> Self::Output {
 *         UserResponse {
 *             id: value.id,
 *             name: value.name,
 *             // password is hidden
 *         }
 *     }
 * }
 * ```
 */
pub trait ResponseSerializer<T>: Send + Sync + 'static {
    type Output: Serialize;

    fn serialize(value: T) -> Self::Output;
}

/**
 * ResponseEnvelope
 *
 * A standard API response wrapper that provides consistent JSON structure
 * across all API responses. Wraps data in a predictable format that includes
 * success status, data payload, and optional metadata.
 *
 * # JSON Structure
 * ```json
 * {
 *   "success": true,
 *   "data": { ... },
 *   "meta": { ... }
 * }
 * ```
 *
 * # Usage
 * ```rust
 * fn get_users() -> ApiResult<Vec<User>> {
 *     Ok(ResponseEnvelope::ok(users))
 * }
 * ```
 */
#[derive(Debug, Clone, Serialize)]
pub struct ResponseEnvelope<T> {
    pub success: bool,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/**
 * Serialized Wrapper
 *
 * A wrapper type that applies a ResponseSerializer to a value.
 * When returned from a controller, it automatically serializes
 * the inner value using the specified serializer.
 *
 * # Type Parameters
 * - `T`: The type being serialized
 * - `S`: The serializer type implementing ResponseSerializer<T>
 */
pub struct Serialized<T, S>
where
    S: ResponseSerializer<T>,
{
    value: T,
    _marker: std::marker::PhantomData<S>,
}

impl<T, S> Serialized<T, S>
where
    S: ResponseSerializer<T>,
{
    pub fn new(value: T) -> Self {
        Self {
            value,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> ResponseEnvelope<T> {
    /// Creates a success response with the given data.
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data,
            meta: None,
        }
    }

    /// Adds metadata to the response.
    pub fn with_meta(mut self, meta: impl Into<serde_json::Value>) -> Self {
        self.meta = Some(meta.into());
        self
    }

    /// Helper for creating paginated responses.
    pub fn paginated(data: T, page: u64, per_page: u64, total: u64) -> Self {
        Self::ok(data).with_meta(json!({
            "page": page,
            "per_page": per_page,
            "total": total,
        }))
    }
}

impl<T> IntoResponse for ResponseEnvelope<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl<T, S> IntoResponse for Serialized<T, S>
where
    S: ResponseSerializer<T>,
{
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(S::serialize(self.value))).into_response()
    }
}

/// A helper type for returning `Result<ResponseEnvelope<T>, HttpException>`.
pub type ApiEnvelopeResult<T> = Result<ResponseEnvelope<T>, HttpException>;

/// A helper type for returning `Result<Serialized<T, S>, HttpException>`.
pub type ApiSerializedResult<T, S> = Result<Serialized<T, S>, HttpException>;
