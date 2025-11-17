mod common;

use common::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_create_user() {
    let ctx = setup_test_db().await;

    let (user_id, actual_username) = create_test_user(&ctx.pool, "testuser", "password123", false)
        .await
        .expect("Failed to create user");

    // Verify user exists in database
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one(
            "SELECT username, email, is_admin FROM users WHERE id = $1",
            &[&user_id],
        )
        .await
        .unwrap();

    let username: String = row.get(0);
    let email: String = row.get(1);
    let is_admin: bool = row.get(2);

    assert_eq!(username, actual_username);
    assert!(username.starts_with("testuser_"));
    assert_eq!(email, format!("{}@test.com", actual_username));
    assert!(!is_admin);
}

#[tokio::test]
#[serial]
async fn test_create_admin_user() {
    let ctx = setup_test_db().await;

    let (user_id, _actual_username) = create_test_user(&ctx.pool, "admin", "admin123", true)
        .await
        .expect("Failed to create admin user");

    // Verify admin flag
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one("SELECT is_admin FROM users WHERE id = $1", &[&user_id])
        .await
        .unwrap();

    let is_admin: bool = row.get(0);
    assert!(is_admin);
}

#[tokio::test]
#[serial]
async fn test_password_is_hashed() {
    let ctx = setup_test_db().await;

    let password = "my_secret_password";
    let (user_id, _actual_username) = create_test_user(&ctx.pool, "testuser", password, false)
        .await
        .expect("Failed to create user");

    // Verify password is hashed (not stored in plain text)
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one("SELECT password_hash FROM users WHERE id = $1", &[&user_id])
        .await
        .unwrap();

    let password_hash: String = row.get(0);

    // Password hash should not equal plain password
    assert_ne!(password_hash, password);

    // Password hash should be bcrypt format (starts with $2)
    assert!(password_hash.starts_with("$2"));

    // Verify password can be validated
    let is_valid = bcrypt::verify(password, &password_hash).unwrap();
    assert!(is_valid);
}

#[tokio::test]
#[serial]
async fn test_username_uniqueness_with_uuid() {
    let ctx = setup_test_db().await;

    // Create first user - will get unique UUID suffix
    let (_user_id1, username1) = create_test_user(&ctx.pool, "testuser", "password123", false)
        .await
        .expect("Failed to create first user");

    // Create second user with same base name - will get different UUID suffix
    let (_user_id2, username2) =
        create_test_user(&ctx.pool, "testuser", "different_password", false)
            .await
            .expect("Failed to create second user");

    // Both should succeed with different usernames due to UUID suffix
    assert!(username1.starts_with("testuser_"));
    assert!(username2.starts_with("testuser_"));
    assert_ne!(username1, username2);
}

#[tokio::test]
#[serial]
async fn test_jwt_token_generation() {
    let ctx = setup_test_db().await;

    let token = create_user_and_login(&ctx.pool, "testuser", "password123")
        .await
        .expect("Failed to create user and login");

    // Token should not be empty
    assert!(!token.is_empty());

    // Token should have JWT format (three parts separated by dots)
    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(parts.len(), 3, "JWT should have 3 parts");
}

#[tokio::test]
#[serial]
async fn test_jwt_token_contains_user_info() {
    use jsonwebtoken::{DecodingKey, Validation, decode};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        user_id: String,
        username: String,
        exp: usize,
        iat: usize,
    }

    let ctx = setup_test_db().await;

    let token = create_user_and_login(&ctx.pool, "testuser", "password123")
        .await
        .expect("Failed to create user and login");

    // Decode token
    let secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "test_secret_key_for_testing".to_string());

    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .expect("Failed to decode token");

    // Verify claims - username will have UUID suffix for uniqueness
    assert!(token_data.claims.username.starts_with("testuser_"));
    assert!(!token_data.claims.user_id.is_empty());
    assert!(token_data.claims.exp > token_data.claims.iat);
}

#[tokio::test]
#[serial]
async fn test_multiple_users_can_login() {
    let ctx = setup_test_db().await;

    let token1 = create_user_and_login(&ctx.pool, "user1", "password1")
        .await
        .expect("Failed to create user1");

    let token2 = create_user_and_login(&ctx.pool, "user2", "password2")
        .await
        .expect("Failed to create user2");

    // Tokens should be different
    assert_ne!(token1, token2);

    // Both tokens should be valid
    assert!(!token1.is_empty());
    assert!(!token2.is_empty());
}
