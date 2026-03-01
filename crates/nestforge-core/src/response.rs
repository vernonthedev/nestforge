use axum::{
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use serde::Serialize;
use serde_json::json;

use crate::HttpException;

pub trait ResponseSerializer<T>: Send + Sync + 'static {
    type Output: Serialize;

    fn serialize(value: T) -> Self::Output;
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseEnvelope<T> {
    pub success: bool,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

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
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data,
            meta: None,
        }
    }

    pub fn with_meta(mut self, meta: impl Into<serde_json::Value>) -> Self {
        self.meta = Some(meta.into());
        self
    }

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

pub type ApiEnvelopeResult<T> = Result<ResponseEnvelope<T>, HttpException>;
pub type ApiSerializedResult<T, S> = Result<Serialized<T, S>, HttpException>;
