//! JWT creation and verification. Lives in `services` (not `handlers`) so that middleware can
//! verify tokens without depending on the handler layer.
//!
//! The signing secret is resolved and validated once at startup (`main.rs`, which may also
//! auto-generate one) and handed over via [`init_secret`]. There is deliberately NO built-in
//! fallback secret: if none is configured, tokens cannot be issued or verified.

use jsonwebtoken::errors::{Error, ErrorKind};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::OnceLock;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub username: String,
    pub exp: usize, // expiration time (required)
    pub iat: usize, // issued at time (for tracking)
}

/// The secret chosen at startup. Set once so that runtime signs/verifies with exactly the secret
/// startup validated or generated, even when an auto-generated one could not be persisted to disk.
static STARTUP_SECRET: OnceLock<String> = OnceLock::new();

/// Register the secret resolved at startup. Later calls are ignored.
pub fn init_secret(secret: String) {
    if STARTUP_SECRET.set(secret).is_err() {
        tracing::warn!("JWT secret was already initialised; ignoring the new value");
    }
}

/// The active secret: the startup secret, else the `JWT_SECRET` environment variable (used when the
/// library is driven without `main`, e.g. integration tests). Empty/blank values count as unset,
/// since an empty HMAC key would let anyone forge tokens.
fn jwt_secret() -> Option<String> {
    STARTUP_SECRET
        .get()
        .cloned()
        .or_else(|| env::var("JWT_SECRET").ok().filter(|s| !s.trim().is_empty()))
}

fn missing_secret() -> Error {
    tracing::error!("JWT secret is not configured - refusing to issue or verify tokens");
    Error::from(ErrorKind::InvalidKeyFormat)
}

fn generate_with(secret: Option<&str>, user_id: &str, username: &str) -> Result<String, Error> {
    let secret = secret.ok_or_else(missing_secret)?;

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
            Error::from(ErrorKind::InvalidToken)
        })?
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_owned(),
        username: username.to_owned(),
        exp: expiration,
        iat,
    };

    let header = Header::new(Algorithm::HS256);
    let key = EncodingKey::from_secret(secret.as_bytes());

    encode(&header, &claims, &key)
}

fn verify_with(secret: Option<&str>, token: &str) -> Result<Claims, Error> {
    let secret = secret.ok_or_else(missing_secret)?;
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);

    decode::<Claims>(token, &key, &validation).map(|data| data.claims)
}

// JWT token utilities
pub fn generate_token(user_id: &str, username: &str) -> Result<String, Error> {
    generate_with(jwt_secret().as_deref(), user_id, username)
}

pub fn verify_token(token: &str) -> Result<Claims, Error> {
    verify_with(jwt_secret().as_deref(), token)
}

#[cfg(test)]
mod tests {
    use super::*;

    const REAL: &str = "a-real-secret-that-is-at-least-32-characters-long";
    // The value the old code silently fell back to when no secret was configured.
    const OLD_SENTINEL: &str = "INVALID_SECRET_NOT_CONFIGURED";

    #[test]
    fn roundtrip_with_a_configured_secret() {
        let token = generate_with(Some(REAL), "user-1", "alice").unwrap();
        let claims = verify_with(Some(REAL), &token).unwrap();
        assert_eq!(claims.sub, "user-1");
        assert_eq!(claims.username, "alice");
    }

    #[test]
    fn no_secret_means_no_tokens_issued_or_accepted() {
        assert!(generate_with(None, "user-1", "alice").is_err());

        let token = generate_with(Some(REAL), "user-1", "alice").unwrap();
        assert!(verify_with(None, &token).is_err());
    }

    #[test]
    fn token_forged_with_the_old_sentinel_is_rejected() {
        // Regression: with no configured secret the app used to sign/verify with this public
        // constant, so anyone could mint a valid token for any user.
        let forged = generate_with(Some(OLD_SENTINEL), "victim-id", "victim").unwrap();
        assert!(verify_with(Some(REAL), &forged).is_err());
        assert!(verify_with(None, &forged).is_err());
    }

    #[test]
    fn a_token_signed_with_a_different_secret_is_rejected() {
        let token = generate_with(Some("another-secret-that-is-32-characters!"), "u", "n").unwrap();
        assert!(verify_with(Some(REAL), &token).is_err());
    }
}
