pub mod auth;

pub use auth::{with_current_user, AuthContext};

// Re-export the error handler from errors module
pub use crate::errors::handle_rejection;
