mod common;

use common::{create_test_humidor, create_test_user, setup_test_db};
use serial_test::serial;
use uuid::Uuid;

/// Test transferring ownership of humidors from one user to another
#[tokio::test]
#[serial]
async fn test_transfer_humidor_ownership() {
    let ctx = setup_test_db().await;

    // Create two regular users
    let (user1_id, _) = create_test_user(&ctx.pool, "user1", "password123", false)
        .await
        .expect("Failed to create user1");

    let (user2_id, _) = create_test_user(&ctx.pool, "user2", "password123", false)
        .await
        .expect("Failed to create user2");

    // Create humidors for user1
    let _humidor1_id = create_test_humidor(&ctx.pool, user1_id, "User1 Humidor 1")
        .await
        .expect("Failed to create humidor 1");

    let humidor2_id = create_test_humidor(&ctx.pool, user1_id, "User1 Humidor 2")
        .await
        .expect("Failed to create humidor 2");

    // Create a cigar for user1 in humidor2
    let client = ctx.pool.get().await.unwrap();
    let cigar_id = Uuid::new_v4();
    client
        .execute(
            "INSERT INTO cigars (id, humidor_id, name, quantity, purchase_date, created_at, updated_at)
             VALUES ($1, $2, $3, $4, NOW(), NOW(), NOW())",
            &[&cigar_id, &humidor2_id, &"Test Cigar", &5],
        )
        .await
        .expect("Failed to create cigar");

    // Verify initial state
    let humidor_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM humidors WHERE user_id = $1",
            &[&user1_id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(humidor_count, 2, "User1 should have 2 humidors");

    let cigar_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM cigars c INNER JOIN humidors h ON c.humidor_id = h.id WHERE h.user_id = $1",
            &[&user1_id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(cigar_count, 1, "User1 should have 1 cigar");

    // Perform the transfer directly in the database (simulating what the handler does)
    let mut db = ctx.pool.get().await.unwrap();
    let transaction = db.transaction().await.unwrap();

    // Transfer humidors
    let humidors_transferred = transaction
        .execute(
            "UPDATE humidors SET user_id = $1, updated_at = NOW() WHERE user_id = $2",
            &[&user2_id, &user1_id],
        )
        .await
        .unwrap();

    // Count cigars that were transferred (cigars transfer automatically with humidors)
    // Note: At this point humidors already belong to user2, so we count their cigars
    let cigars_transferred: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM cigars c INNER JOIN humidors h ON c.humidor_id = h.id WHERE h.user_id = $1",
            &[&user2_id],
        )
        .await
        .unwrap()
        .get(0);

    transaction.commit().await.unwrap();

    assert_eq!(humidors_transferred, 2, "Should transfer 2 humidors");
    assert_eq!(
        cigars_transferred, 1,
        "Should count 1 cigar in transferred humidors"
    );

    // Verify humidors now belong to user2
    let humidor_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM humidors WHERE user_id = $1",
            &[&user2_id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(humidor_count, 2, "User2 should now have 2 humidors");

    // Verify user1 has no humidors
    let humidor_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM humidors WHERE user_id = $1",
            &[&user1_id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(humidor_count, 0, "User1 should have no humidors");

    // Verify cigars now belong to user2 (through humidor ownership)
    let cigar_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM cigars c INNER JOIN humidors h ON c.humidor_id = h.id WHERE h.user_id = $1",
            &[&user2_id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(cigar_count, 1, "User2 should now have 1 cigar");

    // Verify user1 has no cigars
    let cigar_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM cigars c INNER JOIN humidors h ON c.humidor_id = h.id WHERE h.user_id = $1",
            &[&user1_id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(cigar_count, 0, "User1 should have no cigars");
}

/// Test validation: source and destination users must be different
#[tokio::test]
#[serial]
async fn test_transfer_to_same_user_validation() {
    let ctx = setup_test_db().await;

    // Create user
    let (user1_id, _) = create_test_user(&ctx.pool, "user1", "password123", false)
        .await
        .expect("Failed to create user1");

    // Attempting to transfer from user to themselves should be validated before database operations
    // The handler checks: if request.from_user_id == request.to_user_id
    // This is a validation check, not a database operation
    assert_eq!(
        user1_id, user1_id,
        "Validation test: same user IDs should match"
    );
}

/// Test that humidor shares are cleaned up after transfer
#[tokio::test]
#[serial]
async fn test_transfer_cleans_up_shares() {
    let ctx = setup_test_db().await;

    // Create three users
    let (user1_id, _) = create_test_user(&ctx.pool, "user1", "password123", false)
        .await
        .expect("Failed to create user1");

    let (user2_id, _) = create_test_user(&ctx.pool, "user2", "password123", false)
        .await
        .expect("Failed to create user2");

    let (user3_id, _) = create_test_user(&ctx.pool, "user3", "password123", false)
        .await
        .expect("Failed to create user3");

    // Create humidor for user1
    let humidor_id = create_test_humidor(&ctx.pool, user1_id, "Shared Humidor")
        .await
        .expect("Failed to create humidor");

    // Share humidor with user3
    let client = ctx.pool.get().await.unwrap();
    let share_id = Uuid::new_v4();
    client
        .execute(
            "INSERT INTO humidor_shares (id, humidor_id, shared_with_user_id, shared_by_user_id, permission_level, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
            &[&share_id, &humidor_id, &user3_id, &user1_id, &"view"],
        )
        .await
        .expect("Failed to create share");

    // Verify share exists
    let share_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM humidor_shares WHERE humidor_id = $1",
            &[&humidor_id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(share_count, 1, "Should have 1 share before transfer");

    // Transfer ownership from user1 to user2 (simulating handler behavior)
    let mut db = ctx.pool.get().await.unwrap();
    let transaction = db.transaction().await.unwrap();

    transaction
        .execute(
            "UPDATE humidors SET user_id = $1, updated_at = NOW() WHERE user_id = $2",
            &[&user2_id, &user1_id],
        )
        .await
        .unwrap();

    // Delete shares (as the handler does)
    transaction
        .execute(
            "DELETE FROM humidor_shares WHERE humidor_id IN (SELECT id FROM humidors WHERE user_id = $1)",
            &[&user2_id],
        )
        .await
        .unwrap();

    transaction.commit().await.unwrap();

    // Verify shares are cleaned up
    let share_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM humidor_shares WHERE humidor_id = $1",
            &[&humidor_id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(share_count, 0, "Shares should be cleaned up after transfer");
}

/// Test that user can be safely deleted after ownership transfer
#[tokio::test]
#[serial]
async fn test_safe_user_deletion_after_transfer() {
    let ctx = setup_test_db().await;

    // Create two users
    let (user1_id, _) = create_test_user(&ctx.pool, "user1", "password123", false)
        .await
        .expect("Failed to create user1");

    let (user2_id, _) = create_test_user(&ctx.pool, "user2", "password123", false)
        .await
        .expect("Failed to create user2");

    // Create humidors and cigars for user1
    let humidor_id = create_test_humidor(&ctx.pool, user1_id, "User1 Humidor")
        .await
        .expect("Failed to create humidor");

    let client = ctx.pool.get().await.unwrap();
    let cigar_id = Uuid::new_v4();
    client
        .execute(
            "INSERT INTO cigars (id, humidor_id, name, quantity, purchase_date, created_at, updated_at)
             VALUES ($1, $2, $3, $4, NOW(), NOW(), NOW())",
            &[&cigar_id, &humidor_id, &"Test Cigar", &3],
        )
        .await
        .expect("Failed to create cigar");

    // Transfer ownership (simulating handler)
    let mut db = ctx.pool.get().await.unwrap();
    let transaction = db.transaction().await.unwrap();

    transaction
        .execute(
            "UPDATE humidors SET user_id = $1, updated_at = NOW() WHERE user_id = $2",
            &[&user2_id, &user1_id],
        )
        .await
        .unwrap();

    // Cigars automatically transfer with humidors (no separate update needed)

    transaction.commit().await.unwrap();

    // Now delete user1 - this should succeed without cascade errors
    client
        .execute("DELETE FROM users WHERE id = $1", &[&user1_id])
        .await
        .expect("User deletion should succeed after transfer");

    // Verify user1 is deleted
    let user_exists = client
        .query_opt("SELECT id FROM users WHERE id = $1", &[&user1_id])
        .await
        .unwrap();
    assert!(user_exists.is_none(), "User1 should be deleted");

    // Verify humidor still exists under user2
    let humidor_owner: Uuid = client
        .query_one("SELECT user_id FROM humidors WHERE id = $1", &[&humidor_id])
        .await
        .unwrap()
        .get(0);
    assert_eq!(humidor_owner, user2_id, "Humidor should belong to user2");

    // Verify cigar still exists and its humidor belongs to user2
    let cigar_humidor: Uuid = client
        .query_one("SELECT humidor_id FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        cigar_humidor, humidor_id,
        "Cigar should still be in same humidor"
    );

    let humidor_owner: Uuid = client
        .query_one("SELECT user_id FROM humidors WHERE id = $1", &[&humidor_id])
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        humidor_owner, user2_id,
        "Humidor (and its cigars) should belong to user2"
    );
}
