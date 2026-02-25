use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

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
    message: String,
}

/**
* HttpException = framework error type
* This lets controllers return proper HTTP errors without manually building responses.
*/
#[derive(Debug, Clone)]
pub struct HttpException {
    pub status: StatusCode,
    pub message: String,
}

impl HttpException {
    /*
    Generic constructor
    */
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    /*
    Helper constructors (clean DX for controllers)
    */
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
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
            message: self.message,
        };

        (self.status, Json(body)).into_response()
    }
}