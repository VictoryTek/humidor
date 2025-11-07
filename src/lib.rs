// Library exports for integration testing
// This allows tests to access internal modules

pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod services;
pub mod validation;

// Re-export commonly used types
pub use errors::AppError;
pub use middleware::{RateLimiter, AuthContext};
pub use validation::{validate_email, validate_length, validate_positive};

// Export DbPool type for handlers
use deadpool_postgres::Pool;
pub type DbPool = Pool;
