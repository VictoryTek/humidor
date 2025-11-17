pub mod auth;
pub mod metrics;
pub mod rate_limiter;

pub use auth::{AuthContext, with_current_user};
pub use metrics::record_response_metrics;
pub use rate_limiter::RateLimiter;

// Re-export the error handler from errors module
pub use crate::errors::handle_rejection;
