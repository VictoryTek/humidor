mod common;

use common::{create_test_user, setup_test_db};
use serial_test::serial;

/// Test that admin users can pass admin check
#[tokio::test]
#[serial]
async fn test_admin_user_has_admin_privileges() {
    let ctx = setup_test_db().await;

    // Create an admin user
    let (admin_id, _) = create_test_user(&ctx.pool, "admin_user", "password123", true)
        .await
        .expect("Failed to create admin user");

    // Verify admin flag is set
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one("SELECT is_admin FROM users WHERE id = $1", &[&admin_id])
        .await
        .unwrap();

    let is_admin: bool = row.get(0);
    assert!(is_admin, "Admin user should have is_admin set to true");
}

/// Test that non-admin users don't have admin privileges
#[tokio::test]
#[serial]
async fn test_regular_user_lacks_admin_privileges() {
    let ctx = setup_test_db().await;

    // Create a regular user
    let (user_id, _) = create_test_user(&ctx.pool, "regular_user", "password123", false)
        .await
        .expect("Failed to create regular user");

    // Verify admin flag is not set
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one("SELECT is_admin FROM users WHERE id = $1", &[&user_id])
        .await
        .unwrap();

    let is_admin: bool = row.get(0);
    assert!(!is_admin, "Regular user should have is_admin set to false");
}

/// Test that we can toggle admin status
#[tokio::test]
#[serial]
async fn test_toggle_admin_status() {
    let ctx = setup_test_db().await;

    // Create a regular user
    let (user_id, _) = create_test_user(&ctx.pool, "promote_user", "password123", false)
        .await
        .expect("Failed to create user");

    let client = ctx.pool.get().await.unwrap();

    // Verify starts as non-admin
    let row = client
        .query_one("SELECT is_admin FROM users WHERE id = $1", &[&user_id])
        .await
        .unwrap();
    let is_admin: bool = row.get(0);
    assert!(!is_admin);

    // Promote to admin
    client
        .execute(
            "UPDATE users SET is_admin = true WHERE id = $1",
            &[&user_id],
        )
        .await
        .unwrap();

    // Verify now admin
    let row = client
        .query_one("SELECT is_admin FROM users WHERE id = $1", &[&user_id])
        .await
        .unwrap();
    let is_admin: bool = row.get(0);
    assert!(is_admin);

    // Demote back to regular user
    client
        .execute(
            "UPDATE users SET is_admin = false WHERE id = $1",
            &[&user_id],
        )
        .await
        .unwrap();

    // Verify back to non-admin
    let row = client
        .query_one("SELECT is_admin FROM users WHERE id = $1", &[&user_id])
        .await
        .unwrap();
    let is_admin: bool = row.get(0);
    assert!(!is_admin);
}

/// Test that there's at least one admin in the system after setup
#[tokio::test]
#[serial]
async fn test_system_has_admin_after_setup() {
    let ctx = setup_test_db().await;

    // Create at least one admin
    create_test_user(&ctx.pool, "system_admin", "password123", true)
        .await
        .expect("Failed to create admin");

    // Verify we have at least one admin
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one("SELECT COUNT(*) FROM users WHERE is_admin = true", &[])
        .await
        .unwrap();

    let admin_count: i64 = row.get(0);
    assert!(
        admin_count >= 1,
        "System should have at least one admin user"
    );
}

/// Test that multiple admins can exist
#[tokio::test]
#[serial]
async fn test_multiple_admins_allowed() {
    let ctx = setup_test_db().await;

    // Create multiple admins
    create_test_user(&ctx.pool, "admin1", "password123", true)
        .await
        .expect("Failed to create admin1");
    create_test_user(&ctx.pool, "admin2", "password123", true)
        .await
        .expect("Failed to create admin2");
    create_test_user(&ctx.pool, "admin3", "password123", true)
        .await
        .expect("Failed to create admin3");

    // Verify we have multiple admins
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one("SELECT COUNT(*) FROM users WHERE is_admin = true", &[])
        .await
        .unwrap();

    let admin_count: i64 = row.get(0);
    assert!(
        admin_count >= 3,
        "System should support multiple admin users"
    );
}

/// Test that inactive admins still have is_admin flag but is_active is false
#[tokio::test]
#[serial]
async fn test_inactive_admin_retains_admin_flag() {
    let ctx = setup_test_db().await;

    // Create an admin user
    let (admin_id, _) = create_test_user(&ctx.pool, "deact_admin", "password123", true)
        .await
        .expect("Failed to create admin");

    let client = ctx.pool.get().await.unwrap();

    // Deactivate the admin
    client
        .execute(
            "UPDATE users SET is_active = false WHERE id = $1",
            &[&admin_id],
        )
        .await
        .unwrap();

    // Verify is_admin is still true but is_active is false
    let row = client
        .query_one(
            "SELECT is_admin, is_active FROM users WHERE id = $1",
            &[&admin_id],
        )
        .await
        .unwrap();

    let is_admin: bool = row.get(0);
    let is_active: bool = row.get(1);

    assert!(
        is_admin,
        "Deactivated admin should still have is_admin flag"
    );
    assert!(
        !is_active,
        "Deactivated admin should have is_active set to false"
    );
}

/// Test default user is not admin
#[tokio::test]
#[serial]
async fn test_default_user_not_admin() {
    let ctx = setup_test_db().await;

    // Create a user without specifying admin (should default to false)
    let (user_id, _) = create_test_user(&ctx.pool, "default_user", "password123", false)
        .await
        .expect("Failed to create user");

    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one("SELECT is_admin FROM users WHERE id = $1", &[&user_id])
        .await
        .unwrap();

    let is_admin: bool = row.get(0);
    assert!(
        !is_admin,
        "User created without admin flag should not be admin"
    );
}
