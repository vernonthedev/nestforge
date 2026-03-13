use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::Value;

use crate::ValidationErrors;

/**
 * ErrorBody
 *
 * The standard JSON error response structure used by NestForge.
 * Provides consistent error formatting across all HTTP error responses.
 *
 * # JSON Structure
 * ```json
 * {
 *   "statusCode": 500,
 *   "error": "Internal Server Error",
 *   "message": "Something went wrong"
 * }
 * ```
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
 * HttpException
 *
 * The framework's primary error type for HTTP responses.
 * Enables controllers to return proper HTTP error responses
 * without manually constructing response objects.
 *
 * # Fields
 * - `status`: The HTTP status code
 * - `code`: A machine-readable error code for programmatic error handling
 * - `message`: A human-readable error message
 * - `details`: Optional additional error details (often validation errors)
 * - `request_id`: Optional request ID for correlation
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
    /**
     * Generic constructor for creating an HttpException.
     *
     * # Arguments
     * - `status`: The HTTP status code
     * - `code`: A machine-readable error code
     * - `message`: A human-readable error message
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

    /**
     * Constructor that includes additional error details.
     *
     * # Arguments
     * - `status`: The HTTP status code
     * - `code`: A machine-readable error code
     * - `message`: A human-readable error message
     * - `details`: Additional error details (often used for validation errors)
     */
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

    /**
     * Attaches a request ID to the exception for correlation.
     */
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /**
     * Attaches an optional request ID to the exception.
     */
    pub fn with_optional_request_id(mut self, request_id: Option<String>) -> Self {
        self.request_id = request_id;
        self
    }

    /**
     * Creates a 400 Bad Request error.
     */
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "bad_request", message)
    }

    /**
     * Creates a 400 Bad Request error with validation errors.
     */
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

    /**
     * Creates a 401 Unauthorized error.
     */
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "unauthorized", message)
    }

    /**
     * Creates a 403 Forbidden error.
     */
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, "forbidden", message)
    }

    /**
     * Creates a 404 Not Found error.
     */
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", message)
    }

    /**
     * Creates a 500 Internal Server Error.
     */
    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_server_error",
            message,
        )
    }
}

/**
 * IntoResponse Implementation
 *
 * Makes HttpException directly returnable from axum handlers.
 * This enables returning `Result<Json<T>, HttpException>` from controllers
 * and having axum automatically convert the error into a proper HTTP response.
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
