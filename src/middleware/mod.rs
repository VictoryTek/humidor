pub mod auth;

pub use auth::{AuthContext, with_current_user};

// Re-export the error handler from errors module
pub use crate::errors::handle_rejection;