use axum::{
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use serde::Serialize;
use serde_json::json;

use crate::HttpException;

#[derive(Debug, Clone, Serialize)]
pub struct ResponseEnvelope<T> {
    pub success: bool,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
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

pub type ApiEnvelopeResult<T> = Result<ResponseEnvelope<T>, HttpException>;
