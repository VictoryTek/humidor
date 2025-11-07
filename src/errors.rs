use serde::Serialize;
use std::fmt;
use warp::{http::StatusCode, reject::Reject, Reply};

/// Custom error types for the application
#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    DatabaseError(String),
    ValidationError(String),
    Unauthorized,
    Forbidden,
    NotFound(String),
    Conflict(String),
    BadRequest(String),
    InternalServerError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::Unauthorized => write!(f, "Unauthorized"),
            AppError::Forbidden => write!(f, "Forbidden"),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl Reject for AppError {}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<String>>,
}

impl ErrorResponse {
    pub fn new(error: &str, message: &str) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_details(error: &str, message: &str, details: Vec<String>) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
            details: Some(details),
        }
    }
}

impl AppError {
    /// Convert AppError to HTTP response with proper error hiding
    /// Internal errors are logged but never exposed to clients
    pub fn to_http_response(&self) -> (StatusCode, ErrorResponse) {
        match self {
            AppError::ValidationError(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("VALIDATION_FAILED", msg),
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse::new("UNAUTHORIZED", "Authentication required"),
            ),
            AppError::NotFound(resource) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("NOT_FOUND", &format!("{} not found", resource)),
            ),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, ErrorResponse::new("CONFLICT", msg)),
            // Never expose database errors externally
            AppError::DatabaseError(internal_msg) => {
                tracing::error!(
                    error_type = "database_error",
                    error = %internal_msg,
                    "Database error occurred"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::new(
                        "INTERNAL_ERROR",
                        "An error occurred processing your request",
                    ),
                )
            }
            // Never expose internal server errors externally
            AppError::InternalServerError(internal_msg) => {
                tracing::error!(
                    error_type = "internal_server_error",
                    error = %internal_msg,
                    "Internal server error occurred"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::new("INTERNAL_ERROR", "An error occurred"),
                )
            }
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("BAD_REQUEST", msg),
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                ErrorResponse::new("FORBIDDEN", "Access denied"),
            ),
        }
    }
}

/// Convert rejection to HTTP response using standardized error handling
/// This ensures internal errors are never exposed to clients
pub async fn handle_rejection(
    err: warp::Rejection,
) -> Result<impl Reply, std::convert::Infallible> {
    let (status, body) = if let Some(app_err) = err.find::<AppError>() {
        // Use the standardized error converter
        app_err.to_http_response()
    } else if err.is_not_found() {
        (
            StatusCode::NOT_FOUND,
            ErrorResponse::new("NOT_FOUND", "Resource not found"),
        )
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        (
            StatusCode::METHOD_NOT_ALLOWED,
            ErrorResponse::new("METHOD_NOT_ALLOWED", "Method not allowed"),
        )
    } else if let Some(e) = err.find::<warp::body::BodyDeserializeError>() {
        tracing::warn!(
            error_type = "body_deserialize_error",
            error = %e,
            "Failed to deserialize request body"
        );
        (
            StatusCode::BAD_REQUEST,
            ErrorResponse::new("INVALID_BODY", "Invalid request body"),
        )
    } else if err.find::<warp::reject::InvalidQuery>().is_some() {
        tracing::warn!(error_type = "invalid_query", "Invalid query parameters");
        (
            StatusCode::BAD_REQUEST,
            ErrorResponse::new("INVALID_QUERY", "Invalid query parameters"),
        )
    } else if err.find::<warp::reject::MissingHeader>().is_some() {
        (
            StatusCode::BAD_REQUEST,
            ErrorResponse::new("MISSING_HEADER", "Missing required header"),
        )
    } else if err.find::<warp::reject::InvalidHeader>().is_some() {
        (
            StatusCode::BAD_REQUEST,
            ErrorResponse::new("INVALID_HEADER", "Invalid header value"),
        )
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (
            StatusCode::PAYLOAD_TOO_LARGE,
            ErrorResponse::new("PAYLOAD_TOO_LARGE", "Request payload too large"),
        )
    } else if err.find::<warp::reject::UnsupportedMediaType>().is_some() {
        (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ErrorResponse::new("UNSUPPORTED_MEDIA_TYPE", "Unsupported media type"),
        )
    } else {
        // Log unhandled rejections but don't expose details
        tracing::error!(
            error_type = "unhandled_rejection",
            error = ?err,
            "Unhandled rejection occurred"
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorResponse::new("INTERNAL_ERROR", "An error occurred"),
        )
    };

    Ok(warp::reply::with_status(warp::reply::json(&body), status))
}

/// Helper macro to convert database errors
#[macro_export]
macro_rules! db_error {
    ($err:expr) => {
        $crate::errors::AppError::DatabaseError($err.to_string())
    };
}

/// Helper macro for validation errors
#[macro_export]
macro_rules! validation_error {
    ($msg:expr) => {
        $crate::errors::AppError::ValidationError($msg.to_string())
    };
}

/// Helper macro for not found errors
#[macro_export]
macro_rules! not_found {
    ($msg:expr) => {
        $crate::errors::AppError::NotFound($msg.to_string())
    };
}
