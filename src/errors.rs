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

/// Convert AppError to HTTP response
pub async fn handle_rejection(
    err: warp::Rejection,
) -> Result<impl Reply, std::convert::Infallible> {
    let (code, error_type, message) = if err.is_not_found() {
        (
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            "Resource not found".to_string(),
        )
    } else if let Some(app_err) = err.find::<AppError>() {
        match app_err {
            AppError::DatabaseError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                msg.clone(),
            ),
            AppError::ValidationError(msg) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone())
            }
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Authentication required".to_string(),
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Access denied".to_string(),
            ),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone()),
            AppError::InternalServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                msg.clone(),
            ),
        }
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        (
            StatusCode::METHOD_NOT_ALLOWED,
            "METHOD_NOT_ALLOWED",
            "Method not allowed".to_string(),
        )
    } else if err.find::<warp::body::BodyDeserializeError>().is_some() {
        (
            StatusCode::BAD_REQUEST,
            "INVALID_BODY",
            "Invalid request body".to_string(),
        )
    } else {
        eprintln!("Unhandled rejection: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "Internal server error".to_string(),
        )
    };

    let json = warp::reply::json(&ErrorResponse::new(error_type, &message));
    Ok(warp::reply::with_status(json, code))
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
