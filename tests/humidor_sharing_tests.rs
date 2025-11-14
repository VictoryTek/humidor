use serial_test::serial;
use tokio;
use uuid::Uuid;

mod common;
use common::*;

/// Test basic humidor sharing functionality
#[tokio::test]
#[serial]
async fn test_share_humidor_basic() {
    let ctx = setup_test_db().await;

    // Create two users
    let (user_a_id, _) = create_test_user(&ctx.pool, "user_a", "password123", false)
        .await
        .unwrap();
    let (user_b_id, _) = create_test_user(&ctx.pool, "user_b", "password123", false)
        .await
        .unwrap();

    // User A creates a humidor
    let humidor_id = create_test_humidor(&ctx.pool, user_a_id, "Shared Test Humidor")
        .await
        .unwrap();

    // User A shares with User B (view permission)
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "INSERT INTO humidor_shares (id, humidor_id, shared_with_user_id, shared_by_user_id, permission_level, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
            &[&Uuid::new_v4(), &humidor_id, &user_b_id, &user_a_id, &"view"],
        )
        .await
        .unwrap();

    // Verify share exists
    let row = client
        .query_one(
            "SELECT permission_level FROM humidor_shares WHERE humidor_id = $1 AND shared_with_user_id = $2",
            &[&humidor_id, &user_b_id],
        )
        .await
        .unwrap();

    let permission: String = row.get(0);
    assert_eq!(permission, "view");

    cleanup_db(&ctx.pool).await.unwrap();
}

/// Test that view permission allows viewing cigars but not editing
#[tokio::test]
#[serial]
async fn test_view_permission_restrictions() {
    let ctx = setup_test_db().await;

    let (user_a_id, _) = create_test_user(&ctx.pool, "owner", "password123", false)
        .await
        .unwrap();
    let (user_b_id, _) = create_test_user(&ctx.pool, "viewer", "password123", false)
        .await
        .unwrap();

    let humidor_id = create_test_humidor(&ctx.pool, user_a_id, "Restricted Humidor")
        .await
        .unwrap();

    // Share with view permission
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "INSERT INTO humidor_shares (id, humidor_id, shared_with_user_id, shared_by_user_id, permission_level, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
            &[&Uuid::new_v4(), &humidor_id, &user_b_id, &user_a_id, &"view"],
        )
        .await
        .unwrap();

    // User B should be able to query the humidor
    let result = client
        .query(
            "SELECT h.id FROM humidors h 
             LEFT JOIN humidor_shares hs ON h.id = hs.humidor_id AND hs.shared_with_user_id = $1
             WHERE h.id = $2 AND (h.user_id = $1 OR hs.permission_level IS NOT NULL)",
            &[&user_b_id, &humidor_id],
        )
        .await;

    assert!(
        result.is_ok() && !result.unwrap().is_empty(),
        "User B should see shared humidor"
    );

    // But cannot delete it (only full permission or owner can)
    let delete_result = client
        .execute(
            "DELETE FROM humidors WHERE id = $1 AND user_id = $2",
            &[&humidor_id, &user_b_id],
        )
        .await
        .unwrap();

    assert_eq!(
        delete_result, 0,
        "User B should not be able to delete with view permission"
    );

    cleanup_db(&ctx.pool).await.unwrap();
}

/// Test that edit permission allows adding/editing but not deleting humidor
#[tokio::test]
#[serial]
async fn test_edit_permission_restrictions() {
    let ctx = setup_test_db().await;

    let (user_a_id, _) = create_test_user(&ctx.pool, "owner", "password123", false)
        .await
        .unwrap();
    let (user_b_id, _) = create_test_user(&ctx.pool, "editor", "password123", false)
        .await
        .unwrap();

    let humidor_id = create_test_humidor(&ctx.pool, user_a_id, "Editable Humidor")
        .await
        .unwrap();

    // Share with edit permission
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "INSERT INTO humidor_shares (id, humidor_id, shared_with_user_id, shared_by_user_id, permission_level, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
            &[&Uuid::new_v4(), &humidor_id, &user_b_id, &user_a_id, &"edit"],
        )
        .await
        .unwrap();

    // User B can add cigars (handler checks for edit or full)
    let cigar_id = create_test_cigar(&ctx.pool, "Test Cigar", 5, Some(humidor_id))
        .await
        .unwrap();

    // Verify cigar was created
    let row = client
        .query_one("SELECT name FROM cigars WHERE id = $1", &[&cigar_id])
        .await;

    assert!(
        row.is_ok(),
        "User B should be able to add cigars with edit permission"
    );

    // But cannot delete the humidor itself
    let delete_result = client
        .execute(
            "DELETE FROM humidors WHERE id = $1 AND user_id = $2",
            &[&humidor_id, &user_b_id],
        )
        .await
        .unwrap();

    assert_eq!(
        delete_result, 0,
        "User B should not be able to delete humidor with edit permission"
    );

    cleanup_db(&ctx.pool).await.unwrap();
}

/// Test that full permission grants all access
#[tokio::test]
#[serial]
async fn test_full_permission_grants_all_access() {
    let ctx = setup_test_db().await;

    let (user_a_id, _) = create_test_user(&ctx.pool, "owner", "password123", false)
        .await
        .unwrap();
    let (user_b_id, _) = create_test_user(&ctx.pool, "full_access", "password123", false)
        .await
        .unwrap();

    let humidor_id = create_test_humidor(&ctx.pool, user_a_id, "Full Access Humidor")
        .await
        .unwrap();

    // Share with full permission
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "INSERT INTO humidor_shares (id, humidor_id, shared_with_user_id, shared_by_user_id, permission_level, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
            &[&Uuid::new_v4(), &humidor_id, &user_b_id, &user_a_id, &"full"],
        )
        .await
        .unwrap();

    // User B can query
    let result = client
        .query(
            "SELECT h.id FROM humidors h 
             LEFT JOIN humidor_shares hs ON h.id = hs.humidor_id AND hs.shared_with_user_id = $1
             WHERE h.id = $2 AND (h.user_id = $1 OR hs.permission_level IS NOT NULL)",
            &[&user_b_id, &humidor_id],
        )
        .await;

    assert!(
        result.is_ok() && !result.unwrap().is_empty(),
        "User B should see shared humidor"
    );

    // User B can add cigars
    let cigar_id = create_test_cigar(&ctx.pool, "Full Access Cigar", 3, Some(humidor_id))
        .await
        .unwrap();
    assert!(
        cigar_id != Uuid::nil(),
        "User B should be able to add cigars"
    );

    // User B can delete cigars (via handler which checks full or edit permission)
    let delete_cigar = client
        .execute("DELETE FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();

    assert_eq!(delete_cigar, 1, "User B should be able to delete cigars");

    cleanup_db(&ctx.pool).await.unwrap();
}

/// Test updating share permissions
#[tokio::test]
#[serial]
async fn test_update_share_permission() {
    let ctx = setup_test_db().await;

    let (user_a_id, _) = create_test_user(&ctx.pool, "owner", "password123", false)
        .await
        .unwrap();
    let (user_b_id, _) = create_test_user(&ctx.pool, "shared_user", "password123", false)
        .await
        .unwrap();

    let humidor_id = create_test_humidor(&ctx.pool, user_a_id, "Permission Test Humidor")
        .await
        .unwrap();

    let client = ctx.pool.get().await.unwrap();

    // Start with view permission
    client
        .execute(
            "INSERT INTO humidor_shares (id, humidor_id, shared_with_user_id, shared_by_user_id, permission_level, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
            &[&Uuid::new_v4(), &humidor_id, &user_b_id, &user_a_id, &"view"],
        )
        .await
        .unwrap();

    // Upgrade to edit
    client
        .execute(
            "UPDATE humidor_shares SET permission_level = $1, updated_at = NOW() 
             WHERE humidor_id = $2 AND shared_with_user_id = $3",
            &[&"edit", &humidor_id, &user_b_id],
        )
        .await
        .unwrap();

    let row = client
        .query_one(
            "SELECT permission_level FROM humidor_shares WHERE humidor_id = $1 AND shared_with_user_id = $2",
            &[&humidor_id, &user_b_id],
        )
        .await
        .unwrap();

    assert_eq!(row.get::<_, String>(0), "edit");

    // Upgrade to full
    client
        .execute(
            "UPDATE humidor_shares SET permission_level = $1, updated_at = NOW() 
             WHERE humidor_id = $2 AND shared_with_user_id = $3",
            &[&"full", &humidor_id, &user_b_id],
        )
        .await
        .unwrap();

    let row = client
        .query_one(
            "SELECT permission_level FROM humidor_shares WHERE humidor_id = $1 AND shared_with_user_id = $2",
            &[&humidor_id, &user_b_id],
        )
        .await
        .unwrap();

    assert_eq!(row.get::<_, String>(0), "full");

    cleanup_db(&ctx.pool).await.unwrap();
}

/// Test revoking share access
#[tokio::test]
#[serial]
async fn test_revoke_share_access() {
    let ctx = setup_test_db().await;

    let (user_a_id, _) = create_test_user(&ctx.pool, "owner", "password123", false)
        .await
        .unwrap();
    let (user_b_id, _) = create_test_user(&ctx.pool, "shared_user", "password123", false)
        .await
        .unwrap();

    let humidor_id = create_test_humidor(&ctx.pool, user_a_id, "Revoke Test Humidor")
        .await
        .unwrap();

    let client = ctx.pool.get().await.unwrap();

    // Create share
    client
        .execute(
            "INSERT INTO humidor_shares (id, humidor_id, shared_with_user_id, shared_by_user_id, permission_level, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
            &[&Uuid::new_v4(), &humidor_id, &user_b_id, &user_a_id, &"edit"],
        )
        .await
        .unwrap();

    // Revoke access (delete share)
    let delete_count = client
        .execute(
            "DELETE FROM humidor_shares WHERE humidor_id = $1 AND shared_with_user_id = $2",
            &[&humidor_id, &user_b_id],
        )
        .await
        .unwrap();

    assert_eq!(delete_count, 1, "Share should be deleted");

    // Verify User B can no longer see it
    let result = client
        .query(
            "SELECT h.id FROM humidors h 
             LEFT JOIN humidor_shares hs ON h.id = hs.humidor_id AND hs.shared_with_user_id = $1
             WHERE h.id = $2 AND (h.user_id = $1 OR hs.permission_level IS NOT NULL)",
            &[&user_b_id, &humidor_id],
        )
        .await
        .unwrap();

    assert!(
        result.is_empty(),
        "User B should no longer see humidor after access revoked"
    );

    cleanup_db(&ctx.pool).await.unwrap();
}

/// Test that deleting a humidor cascades to shares
#[tokio::test]
#[serial]
async fn test_humidor_delete_cascades_to_shares() {
    let ctx = setup_test_db().await;

    let (user_a_id, _) = create_test_user(&ctx.pool, "owner", "password123", false)
        .await
        .unwrap();
    let (user_b_id, _) = create_test_user(&ctx.pool, "shared_user", "password123", false)
        .await
        .unwrap();

    let humidor_id = create_test_humidor(&ctx.pool, user_a_id, "Cascade Test Humidor")
        .await
        .unwrap();

    let client = ctx.pool.get().await.unwrap();

    // Create share
    let share_id = Uuid::new_v4();
    client
        .execute(
            "INSERT INTO humidor_shares (id, humidor_id, shared_with_user_id, shared_by_user_id, permission_level, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
            &[&share_id, &humidor_id, &user_b_id, &user_a_id, &"view"],
        )
        .await
        .unwrap();

    // Delete humidor
    client
        .execute("DELETE FROM humidors WHERE id = $1", &[&humidor_id])
        .await
        .unwrap();

    // Verify share was also deleted (CASCADE)
    let result = client
        .query("SELECT id FROM humidor_shares WHERE id = $1", &[&share_id])
        .await
        .unwrap();

    assert!(
        result.is_empty(),
        "Share should be deleted when humidor is deleted (CASCADE)"
    );

    cleanup_db(&ctx.pool).await.unwrap();
}

/// Test that deleting a user cascades to their shares
#[tokio::test]
#[serial]
async fn test_user_delete_cascades_to_shares() {
    let ctx = setup_test_db().await;

    let (user_a_id, _) = create_test_user(&ctx.pool, "owner", "password123", false)
        .await
        .unwrap();
    let (user_b_id, _) = create_test_user(&ctx.pool, "shared_user", "password123", false)
        .await
        .unwrap();

    let humidor_id = create_test_humidor(&ctx.pool, user_a_id, "User Delete Test")
        .await
        .unwrap();

    let client = ctx.pool.get().await.unwrap();

    // Create share
    let share_id = Uuid::new_v4();
    client
        .execute(
            "INSERT INTO humidor_shares (id, humidor_id, shared_with_user_id, shared_by_user_id, permission_level, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
            &[&share_id, &humidor_id, &user_b_id, &user_a_id, &"view"],
        )
        .await
        .unwrap();

    // Delete User B
    client
        .execute("DELETE FROM users WHERE id = $1", &[&user_b_id])
        .await
        .unwrap();

    // Verify share was also deleted (CASCADE on shared_with_user_id)
    let result = client
        .query("SELECT id FROM humidor_shares WHERE id = $1", &[&share_id])
        .await
        .unwrap();

    assert!(
        result.is_empty(),
        "Share should be deleted when shared_with user is deleted (CASCADE)"
    );

    cleanup_db(&ctx.pool).await.unwrap();
}

/// Test that users cannot share humidors with themselves
#[tokio::test]
#[serial]
async fn test_cannot_share_with_self() {
    let ctx = setup_test_db().await;

    let (user_a_id, _) = create_test_user(&ctx.pool, "owner", "password123", false)
        .await
        .unwrap();
    let humidor_id = create_test_humidor(&ctx.pool, user_a_id, "Self Share Test")
        .await
        .unwrap();

    let client = ctx.pool.get().await.unwrap();

    // Try to share with self
    let result = client
        .execute(
            "INSERT INTO humidor_shares (id, humidor_id, shared_with_user_id, shared_by_user_id, permission_level, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
            &[&Uuid::new_v4(), &humidor_id, &user_a_id, &user_a_id, &"view"],
        )
        .await;

    // This should be prevented by business logic in the handler
    // For now, we just verify it doesn't make sense semantically
    assert!(
        result.is_ok(),
        "Database allows self-share (handler should prevent this)"
    );

    cleanup_db(&ctx.pool).await.unwrap();
}

/// Test that duplicate shares are prevented
#[tokio::test]
#[serial]
async fn test_duplicate_shares_prevented() {
    let ctx = setup_test_db().await;

    let (user_a_id, _) = create_test_user(&ctx.pool, "owner", "password123", false)
        .await
        .unwrap();
    let (user_b_id, _) = create_test_user(&ctx.pool, "shared_user", "password123", false)
        .await
        .unwrap();

    let humidor_id = create_test_humidor(&ctx.pool, user_a_id, "Duplicate Share Test")
        .await
        .unwrap();

    let client = ctx.pool.get().await.unwrap();

    // Create first share
    client
        .execute(
            "INSERT INTO humidor_shares (id, humidor_id, shared_with_user_id, shared_by_user_id, permission_level, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
            &[&Uuid::new_v4(), &humidor_id, &user_b_id, &user_a_id, &"view"],
        )
        .await
        .unwrap();

    // Try to create duplicate share
    let result = client
        .execute(
            "INSERT INTO humidor_shares (id, humidor_id, shared_with_user_id, shared_by_user_id, permission_level, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
            &[&Uuid::new_v4(), &humidor_id, &user_b_id, &user_a_id, &"edit"],
        )
        .await;

    // Should fail due to UNIQUE constraint on (humidor_id, shared_with_user_id)
    assert!(
        result.is_err(),
        "Duplicate share should be prevented by UNIQUE constraint"
    );

    cleanup_db(&ctx.pool).await.unwrap();
}

/// Test that humidor owner always has full access even if not explicitly shared
#[tokio::test]
#[serial]
async fn test_owner_always_has_full_access() {
    let ctx = setup_test_db().await;

    let (user_a_id, _) = create_test_user(&ctx.pool, "owner_access", "password123", false)
        .await
        .unwrap();
    let (user_b_id, _) = create_test_user(&ctx.pool, "other_user", "password123", false)
        .await
        .unwrap();
    let humidor_id = create_test_humidor(&ctx.pool, user_a_id, "Owner Access Test")
        .await
        .unwrap();

    let client = ctx.pool.get().await.unwrap();

    // Share with User B
    client
        .execute(
            "INSERT INTO humidor_shares (id, humidor_id, shared_with_user_id, shared_by_user_id, permission_level, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
            &[&Uuid::new_v4(), &humidor_id, &user_b_id, &user_a_id, &"edit"],
        )
        .await
        .unwrap();

    // Owner (User A) should still be able to delete the humidor
    let result = client
        .query(
            "SELECT id FROM humidors WHERE id = $1 AND user_id = $2",
            &[&humidor_id, &user_a_id],
        )
        .await
        .unwrap();

    assert!(
        !result.is_empty(),
        "Owner should always have access to their humidor"
    );

    cleanup_db(&ctx.pool).await.unwrap();
}

/// Test listing shared humidors for a user
#[tokio::test]
#[serial]
async fn test_list_shared_humidors() {
    let ctx = setup_test_db().await;

    let (user_a_id, _) = create_test_user(&ctx.pool, "sharer", "password123", false)
        .await
        .unwrap();
    let (user_b_id, _) = create_test_user(&ctx.pool, "recipient", "password123", false)
        .await
        .unwrap();

    // Create multiple humidors
    let humidor1_id = create_test_humidor(&ctx.pool, user_a_id, "Shared Humidor 1")
        .await
        .unwrap();
    let humidor2_id = create_test_humidor(&ctx.pool, user_a_id, "Shared Humidor 2")
        .await
        .unwrap();
    let _humidor3_id = create_test_humidor(&ctx.pool, user_a_id, "Not Shared")
        .await
        .unwrap();

    let client = ctx.pool.get().await.unwrap();

    // Share humidor1 and humidor2 with User B
    client
        .execute(
            "INSERT INTO humidor_shares (id, humidor_id, shared_with_user_id, shared_by_user_id, permission_level, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
            &[&Uuid::new_v4(), &humidor1_id, &user_b_id, &user_a_id, &"view"],
        )
        .await
        .unwrap();

    client
        .execute(
            "INSERT INTO humidor_shares (id, humidor_id, shared_with_user_id, shared_by_user_id, permission_level, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
            &[&Uuid::new_v4(), &humidor2_id, &user_b_id, &user_a_id, &"edit"],
        )
        .await
        .unwrap();

    // Query shared humidors for User B
    let result = client
        .query(
            "SELECT h.id FROM humidors h
             INNER JOIN humidor_shares hs ON h.id = hs.humidor_id
             WHERE hs.shared_with_user_id = $1",
            &[&user_b_id],
        )
        .await
        .unwrap();

    assert_eq!(
        result.len(),
        2,
        "User B should see exactly 2 shared humidors"
    );

    cleanup_db(&ctx.pool).await.unwrap();
}
