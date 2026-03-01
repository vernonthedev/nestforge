use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::Value;

use crate::ValidationErrors;

/**
* ErrorBody = standard JSON error response shape.
*
* Keeping this simple and clean for now:
* {
*   "statusCode": 500,
*   "error": "Internal Server Error",
*   "message": "Something went wrong"
* }
*/
#[derive(Serialize)]
struct ErrorBody {
    #[serde(rename = "statusCode")]
    status_code: u16,
    error: String,
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Value>,
    #[serde(rename = "requestId", skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,
}

/**
* HttpException = framework error type
* This lets controllers return proper HTTP errors without manually building responses.
*/
#[derive(Debug, Clone)]
pub struct HttpException {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
    pub details: Option<Value>,
    pub request_id: Option<String>,
}

impl HttpException {
    /*
    Generic constructor
    */
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            details: None,
            request_id: None,
        }
    }

    pub fn with_details(
        status: StatusCode,
        code: &'static str,
        message: impl Into<String>,
        details: Value,
    ) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            details: Some(details),
            request_id: None,
        }
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    pub fn with_optional_request_id(mut self, request_id: Option<String>) -> Self {
        self.request_id = request_id;
        self
    }

    /*
    Helper constructors (clean DX for controllers)
    */
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "bad_request", message)
    }

    pub fn bad_request_validation(errors: ValidationErrors) -> Self {
        let message = "Validation failed".to_string();
        let details = serde_json::to_value(errors).unwrap_or(Value::Null);
        Self::with_details(
            StatusCode::BAD_REQUEST,
            "validation_failed",
            message,
            details,
        )
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "unauthorized", message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, "forbidden", message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", message)
    }

    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_server_error",
            message,
        )
    }
}

/**
* IntoResponse makes HttpException directly returnable from axum handlers.
*
* So handlers can return:
* Result<Json<T>, HttpException>
* and axum knows how to turn the error into a real HTTP response.
*/
impl IntoResponse for HttpException {
    fn into_response(self) -> Response {
        let error_name = self
            .status
            .canonical_reason()
            .unwrap_or("Error")
            .to_string();

        let body = ErrorBody {
            status_code: self.status.as_u16(),
            error: error_name,
            code: self.code,
            message: self.message,
            details: self.details,
            request_id: self.request_id,
        };

        (self.status, Json(body)).into_response()
    }
}
