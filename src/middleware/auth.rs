use crate::DbPool;
use crate::errors::AppError;
use crate::handlers::auth::verify_token;
use crate::models::UserResponse;
use std::convert::Infallible;
use uuid::Uuid;
use warp::{Filter, Rejection, reject};

// Authentication context that gets passed to handlers
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    #[allow(dead_code)]
    pub username: String,
    pub user: Option<UserResponse>,
}

impl AuthContext {
    pub fn new(user_id: Uuid, username: String) -> Self {
        Self {
            user_id,
            username,
            user: None,
        }
    }

    pub fn with_user(mut self, user: UserResponse) -> Self {
        self.user = Some(user);
        self
    }

    /// Check if the current user is an admin
    pub fn is_admin(&self) -> bool {
        self.user.as_ref().map(|u| u.is_admin).unwrap_or(false)
    }

    /// Get a reference to the user data if available
    #[allow(dead_code)]
    pub fn get_user(&self) -> Result<&UserResponse, AppError> {
        self.user.as_ref().ok_or(AppError::Unauthorized)
    }
}

// Extract token from Authorization header or cookie
fn extract_token_from_headers(headers: &warp::http::HeaderMap) -> Option<String> {
    // First, try Authorization header
    if let Some(auth_header) = headers.get(warp::http::header::AUTHORIZATION)
        && let Ok(auth_str) = auth_header.to_str()
        && let Some(stripped) = auth_str.strip_prefix("Bearer ")
    {
        return Some(stripped.to_string());
    }

    // Then try cookie
    if let Some(cookie_header) = headers.get(warp::http::header::COOKIE)
        && let Ok(cookie_str) = cookie_header.to_str()
    {
        for cookie in cookie_str.split(';') {
            let cookie = cookie.trim();
            if let Some(token) = cookie.strip_prefix("auth_token=") {
                return Some(token.to_string());
            }
        }
    }

    None
}

// Middleware that extracts and validates JWT token
pub fn with_auth() -> impl Filter<Extract = (AuthContext,), Error = Rejection> + Clone {
    warp::header::headers_cloned().and_then(|headers: warp::http::HeaderMap| async move {
        let token = extract_token_from_headers(&headers)
            .ok_or_else(|| reject::custom(AppError::Unauthorized))?;

        let claims = verify_token(&token).map_err(|_| reject::custom(AppError::Unauthorized))?;

        let user_id =
            Uuid::parse_str(&claims.sub).map_err(|_| reject::custom(AppError::Unauthorized))?;

        Ok::<AuthContext, Rejection>(AuthContext::new(user_id, claims.username))
    })
}

// Middleware that includes user data from database
pub fn with_current_user(
    pool: DbPool,
) -> impl Filter<Extract = (AuthContext,), Error = Rejection> + Clone {
    with_auth()
        .and(warp::any().map(move || pool.clone()))
        .and_then(|auth_ctx: AuthContext, pool: DbPool| async move {
            // Get connection from pool
            let db = match pool.get().await {
                Ok(conn) => conn,
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        user_id = %auth_ctx.user_id,
                        "Failed to get database connection in auth middleware"
                    );
                    return Err(reject::custom(AppError::Unauthorized));
                }
            };

            // Fetch user data from database
            let query = "
                SELECT id, username, email, full_name, is_admin, is_active, created_at, updated_at
                FROM users 
                WHERE id = $1 AND is_active = true
            ";

            match db.query_opt(query, &[&auth_ctx.user_id]).await {
                Ok(Some(row)) => {
                    let user = UserResponse {
                        id: row.get("id"),
                        username: row.get("username"),
                        email: row.get("email"),
                        full_name: row.get("full_name"),
                        is_admin: row.get("is_admin"),
                        is_active: row.get("is_active"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                    };

                    Ok(auth_ctx.with_user(user))
                }
                Ok(None) => Err(reject::custom(AppError::Unauthorized)),
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        user_id = %auth_ctx.user_id,
                        "Database error in auth middleware"
                    );
                    Err(reject::custom(AppError::Unauthorized))
                }
            }
        })
}

// Optional auth that doesn't fail if no token is present
#[allow(dead_code)]
pub fn with_optional_auth()
-> impl Filter<Extract = (Option<AuthContext>,), Error = Infallible> + Clone {
    warp::header::headers_cloned().map(|headers: warp::http::HeaderMap| {
        let token = match extract_token_from_headers(&headers) {
            Some(token) => token,
            None => return None,
        };

        match verify_token(&token) {
            Ok(claims) => match Uuid::parse_str(&claims.sub) {
                Ok(user_id) => Some(AuthContext::new(user_id, claims.username)),
                Err(_) => None,
            },
            Err(_) => None,
        }
    })
}

// Combined middleware for admin routes: auth + user data + admin check
// Use this for endpoints that require admin privileges
pub fn with_admin(
    pool: DbPool,
) -> impl Filter<Extract = (AuthContext,), Error = Rejection> + Clone {
    with_current_user(pool).and_then(|auth_ctx: AuthContext| async move {
        if auth_ctx.is_admin() {
            Ok(auth_ctx)
        } else {
            tracing::warn!(
                user_id = %auth_ctx.user_id,
                username = %auth_ctx.username,
                "Non-admin user attempted to access admin endpoint"
            );
            Err(reject::custom(AppError::Forbidden(
                "Admin privileges required".to_string(),
            )))
        }
    })
}
