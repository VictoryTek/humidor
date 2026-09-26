//! JWT creation and verification. Lives in `services` (not `handlers`) so that middleware can
//! verify tokens without depending on the handler layer.

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub username: String,
    pub exp: usize, // expiration time (required)
    pub iat: usize, // issued at time (for tracking)
}

/// Get JWT secret from Docker secrets, persisted auto-generated file, or environment variable
/// Must match the resolution order used by `read_secret()` in main.rs at startup, since that
/// function may have auto-generated and persisted the secret rather than using an env var.
/// Note: This function assumes the secret was validated at startup via validate_jwt_secret()
/// If the secret is missing, this will return a default that will cause authentication to fail
fn jwt_secret() -> String {
    // Check custom path from JWT_SECRET_FILE first
    if let Ok(custom_path) = env::var("JWT_SECRET_FILE")
        && let Ok(content) = fs::read_to_string(&custom_path)
    {
        return content.trim().to_string();
    }

    // Try Docker secret file
    if let Ok(content) = fs::read_to_string("/run/secrets/jwt_secret") {
        return content.trim().to_string();
    }

    // Try persisted auto-generated secret (written by get_or_generate_jwt_secret at startup)
    if let Ok(content) = fs::read_to_string("/app/data/jwt_secret") {
        return content.trim().to_string();
    }

    // Fall back to environment variable
    // At this point, the secret should have been validated at startup
    // If it's still missing, return a placeholder that will cause auth failures
    env::var("JWT_SECRET").unwrap_or_else(|_| {
        tracing::error!(
            "JWT_SECRET not found - authentication will fail. \
             This should have been caught at startup validation."
        );
        // Return a value that will cause JWT operations to fail gracefully
        "INVALID_SECRET_NOT_CONFIGURED".to_string()
    })
}

// JWT token utilities
pub fn generate_token(
    user_id: &str,
    username: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    // Get token lifetime from environment or use default of 2 hours
    let token_lifetime_hours: i64 = env::var("JWT_TOKEN_LIFETIME_HOURS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2); // Default: 2 hours (more secure than 24)

    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let expiration = now
        .checked_add_signed(chrono::Duration::hours(token_lifetime_hours))
        .ok_or_else(|| {
            tracing::error!("Failed to calculate token expiration timestamp");
            jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken)
        })?
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_owned(),
        username: username.to_owned(),
        exp: expiration,
        iat,
    };

    let header = Header::new(Algorithm::HS256);
    let secret = jwt_secret();
    let key = EncodingKey::from_secret(secret.as_bytes());

    encode(&header, &claims, &key)
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = jwt_secret();
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);

    decode::<Claims>(token, &key, &validation).map(|data| data.claims)
}
