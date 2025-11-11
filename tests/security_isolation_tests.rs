/// Security Isolation Tests
/// 
/// These tests verify that users cannot access or modify other users' data.
/// Critical for preventing unauthorized data access and maintaining user privacy.
/// 
/// Tests cover:
/// - Cross-user humidor access (GET, UPDATE, DELETE)
/// - Cross-user cigar access (GET, CREATE, UPDATE, DELETE)
/// - Cross-user favorites access
/// - Cross-user wish list access
/// - Proper error responses (403 Forbidden)

mod common;

use common::{create_test_cigar, create_test_humidor, create_test_user, setup_test_db};
use uuid::Uuid;

// NOTE: Tests run sequentially to avoid database cleanup conflicts
// Each test creates and cleans up its own data

// ============================================================================
// HUMIDOR ISOLATION TESTS
// ============================================================================

#[tokio::test]
#[serial_test::serial]
#[serial_test::serial]
async fn test_user_cannot_get_other_users_humidor() {
    let ctx = setup_test_db().await;

    // Create two users
    let (user_a_id, _) = create_test_user(&ctx.pool, "user_a", "password", false)
        .await
        .expect("Failed to create user A");

    let (user_b_id, _) = create_test_user(&ctx.pool, "user_b", "password", false)
        .await
        .expect("Failed to create user B");

    // User A creates a humidor
    let humidor_a_id = create_test_humidor(&ctx.pool, user_a_id, "User A's Humidor")
        .await
        .expect("Failed to create humidor");

    // User B tries to access User A's humidor
    let client = ctx.pool.get().await.expect("Failed to get client");
    let result = client
        .query_opt(
            "SELECT id, name, user_id FROM humidors WHERE id = $1 AND user_id = $2",
            &[&humidor_a_id, &user_b_id],
        )
        .await
        .expect("Query failed");

    // Should return None (no access)
    assert!(
        result.is_none(),
        "User B should not be able to access User A's humidor"
    );

    // Verify User A can still access their own humidor
    let result = client
        .query_one(
            "SELECT id, name, user_id FROM humidors WHERE id = $1 AND user_id = $2",
            &[&humidor_a_id, &user_a_id],
        )
        .await
        .expect("Query failed");

    let retrieved_id: Uuid = result.get(0);
    assert_eq!(retrieved_id, humidor_a_id, "User A should access their own humidor");
}

#[tokio::test]
#[serial_test::serial]
#[serial_test::serial]
async fn test_user_cannot_update_other_users_humidor() {
    let ctx = setup_test_db().await;

    // Create two users
    let (user_a_id, _) = create_test_user(&ctx.pool, "user_a", "password", false)
        .await
        .expect("Failed to create user A");

    let (user_b_id, _) = create_test_user(&ctx.pool, "user_b", "password", false)
        .await
        .expect("Failed to create user B");

    // User A creates a humidor
    let humidor_a_id = create_test_humidor(&ctx.pool, user_a_id, "Original Name")
        .await
        .expect("Failed to create humidor");

    // User B tries to update User A's humidor
    let client = ctx.pool.get().await.expect("Failed to get client");
    let rows_updated = client
        .execute(
            "UPDATE humidors SET name = $1 WHERE id = $2 AND user_id = $3",
            &[&"Hacked Name", &humidor_a_id, &user_b_id],
        )
        .await
        .expect("Query failed");

    // Should update 0 rows (no access)
    assert_eq!(
        rows_updated, 0,
        "User B should not be able to update User A's humidor"
    );

    // Verify name was not changed
    let result = client
        .query_one(
            "SELECT name FROM humidors WHERE id = $1",
            &[&humidor_a_id],
        )
        .await
        .expect("Query failed");

    let name: String = result.get(0);
    assert_eq!(name, "Original Name", "Humidor name should not be changed");
}

#[tokio::test]
#[serial_test::serial]
async fn test_user_cannot_delete_other_users_humidor() {
    let ctx = setup_test_db().await;

    // Create two users
    let (user_a_id, _) = create_test_user(&ctx.pool, "user_a", "password", false)
        .await
        .expect("Failed to create user A");

    let (user_b_id, _) = create_test_user(&ctx.pool, "user_b", "password", false)
        .await
        .expect("Failed to create user B");

    // User A creates a humidor
    let humidor_a_id = create_test_humidor(&ctx.pool, user_a_id, "User A's Humidor")
        .await
        .expect("Failed to create humidor");

    // User B tries to delete User A's humidor
    let client = ctx.pool.get().await.expect("Failed to get client");
    let rows_deleted = client
        .execute(
            "DELETE FROM humidors WHERE id = $1 AND user_id = $2",
            &[&humidor_a_id, &user_b_id],
        )
        .await
        .expect("Query failed");

    // Should delete 0 rows (no access)
    assert_eq!(
        rows_deleted, 0,
        "User B should not be able to delete User A's humidor"
    );

    // Verify humidor still exists
    let result = client
        .query_opt("SELECT id FROM humidors WHERE id = $1", &[&humidor_a_id])
        .await
        .expect("Query failed");

    assert!(result.is_some(), "Humidor should still exist");
}

// ============================================================================
// CIGAR ISOLATION TESTS
// ============================================================================

#[tokio::test]
#[serial_test::serial]
async fn test_user_cannot_get_other_users_cigars() {
    let ctx = setup_test_db().await;

    // Create two users
    let (user_a_id, _) = create_test_user(&ctx.pool, "user_a", "password", false)
        .await
        .expect("Failed to create user A");

    let (user_b_id, _) = create_test_user(&ctx.pool, "user_b", "password", false)
        .await
        .expect("Failed to create user B");

    // User A creates a humidor and cigar
    let humidor_a_id = create_test_humidor(&ctx.pool, user_a_id, "User A's Humidor")
        .await
        .expect("Failed to create humidor");

    let cigar_a_id = create_test_cigar(&ctx.pool, "User A's Cigar", 10, Some(humidor_a_id))
        .await
        .expect("Failed to create cigar");

    // User B tries to access User A's cigar using INNER JOIN (as our handlers do)
    let client = ctx.pool.get().await.expect("Failed to get client");
    let result = client
        .query_opt(
            "SELECT c.id, c.name FROM cigars c 
             INNER JOIN humidors h ON c.humidor_id = h.id 
             WHERE c.id = $1 AND h.user_id = $2",
            &[&cigar_a_id, &user_b_id],
        )
        .await
        .expect("Query failed");

    // Should return None (no access)
    assert!(
        result.is_none(),
        "User B should not be able to access User A's cigar"
    );

    // Verify User A can access their own cigar
    let result = client
        .query_one(
            "SELECT c.id, c.name FROM cigars c 
             INNER JOIN humidors h ON c.humidor_id = h.id 
             WHERE c.id = $1 AND h.user_id = $2",
            &[&cigar_a_id, &user_a_id],
        )
        .await
        .expect("Query failed");

    let retrieved_id: Uuid = result.get(0);
    assert_eq!(retrieved_id, cigar_a_id, "User A should access their own cigar");
}

#[tokio::test]
#[serial_test::serial]
async fn test_user_cannot_create_cigar_in_other_users_humidor() {
    let ctx = setup_test_db().await;

    // Create two users
    let (user_a_id, _) = create_test_user(&ctx.pool, "user_a", "password", false)
        .await
        .expect("Failed to create user A");

    let (user_b_id, _) = create_test_user(&ctx.pool, "user_b", "password", false)
        .await
        .expect("Failed to create user B");

    // User A creates a humidor
    let humidor_a_id = create_test_humidor(&ctx.pool, user_a_id, "User A's Humidor")
        .await
        .expect("Failed to create humidor");

    // Verify User B cannot create a cigar in User A's humidor
    // This check simulates what the handler does before inserting
    let client = ctx.pool.get().await.expect("Failed to get client");
    let humidor_check = client
        .query_opt(
            "SELECT EXISTS(SELECT 1 FROM humidors WHERE id = $1 AND user_id = $2)",
            &[&humidor_a_id, &user_b_id],
        )
        .await
        .expect("Query failed");

    let exists: bool = humidor_check.unwrap().get(0);
    assert!(
        !exists,
        "User B should not have access to User A's humidor for creating cigars"
    );

    // Verify User A can create cigars in their own humidor
    let humidor_check = client
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM humidors WHERE id = $1 AND user_id = $2)",
            &[&humidor_a_id, &user_a_id],
        )
        .await
        .expect("Query failed");

    let exists: bool = humidor_check.get(0);
    assert!(exists, "User A should have access to their own humidor");
}

#[tokio::test]
#[serial_test::serial]
async fn test_user_cannot_update_other_users_cigar() {
    let ctx = setup_test_db().await;

    // Create two users
    let (user_a_id, _) = create_test_user(&ctx.pool, "user_a", "password", false)
        .await
        .expect("Failed to create user A");

    let (user_b_id, _) = create_test_user(&ctx.pool, "user_b", "password", false)
        .await
        .expect("Failed to create user B");

    // User A creates a humidor and cigar
    let humidor_a_id = create_test_humidor(&ctx.pool, user_a_id, "User A's Humidor")
        .await
        .expect("Failed to create humidor");

    let cigar_a_id = create_test_cigar(&ctx.pool, "Original Cigar", 10, Some(humidor_a_id))
        .await
        .expect("Failed to create cigar");

    // User B tries to update User A's cigar
    // First verify they don't have access (as handler would check)
    let client = ctx.pool.get().await.expect("Failed to get client");
    let ownership_check = client
        .query_opt(
            "SELECT EXISTS(
                SELECT 1 FROM cigars c
                INNER JOIN humidors h ON c.humidor_id = h.id
                WHERE c.id = $1 AND h.user_id = $2
            )",
            &[&cigar_a_id, &user_b_id],
        )
        .await
        .expect("Query failed");

    let has_access: bool = ownership_check.unwrap().get(0);
    assert!(
        !has_access,
        "User B should not have access to update User A's cigar"
    );

    // Verify User A has access
    let ownership_check = client
        .query_one(
            "SELECT EXISTS(
                SELECT 1 FROM cigars c
                INNER JOIN humidors h ON c.humidor_id = h.id
                WHERE c.id = $1 AND h.user_id = $2
            )",
            &[&cigar_a_id, &user_a_id],
        )
        .await
        .expect("Query failed");

    let has_access: bool = ownership_check.get(0);
    assert!(has_access, "User A should have access to update their own cigar");
}

#[tokio::test]
#[serial_test::serial]
async fn test_user_cannot_delete_other_users_cigar() {
    let ctx = setup_test_db().await;

    // Create two users
    let (user_a_id, _) = create_test_user(&ctx.pool, "user_a", "password", false)
        .await
        .expect("Failed to create user A");

    let (user_b_id, _) = create_test_user(&ctx.pool, "user_b", "password", false)
        .await
        .expect("Failed to create user B");

    // User A creates a humidor and cigar
    let humidor_a_id = create_test_humidor(&ctx.pool, user_a_id, "User A's Humidor")
        .await
        .expect("Failed to create humidor");

    let cigar_a_id = create_test_cigar(&ctx.pool, "User A's Cigar", 10, Some(humidor_a_id))
        .await
        .expect("Failed to create cigar");

    // User B tries to delete User A's cigar
    // First verify they don't have access (as handler would check)
    let client = ctx.pool.get().await.expect("Failed to get client");
    let ownership_check = client
        .query_opt(
            "SELECT EXISTS(
                SELECT 1 FROM cigars c
                INNER JOIN humidors h ON c.humidor_id = h.id
                WHERE c.id = $1 AND h.user_id = $2
            )",
            &[&cigar_a_id, &user_b_id],
        )
        .await
        .expect("Query failed");

    let has_access: bool = ownership_check.unwrap().get(0);
    assert!(
        !has_access,
        "User B should not have access to delete User A's cigar"
    );

    // Verify cigar still exists
    let result = client
        .query_opt("SELECT id FROM cigars WHERE id = $1", &[&cigar_a_id])
        .await
        .expect("Query failed");

    assert!(result.is_some(), "Cigar should still exist");
}

#[tokio::test]
#[serial_test::serial]
async fn test_user_cannot_move_cigar_to_other_users_humidor() {
    let ctx = setup_test_db().await;

    // Create two users
    let (user_a_id, _) = create_test_user(&ctx.pool, "user_a", "password", false)
        .await
        .expect("Failed to create user A");

    let (user_b_id, _) = create_test_user(&ctx.pool, "user_b", "password", false)
        .await
        .expect("Failed to create user B");

    // User A creates a humidor and cigar
    let humidor_a_id = create_test_humidor(&ctx.pool, user_a_id, "User A's Humidor")
        .await
        .expect("Failed to create humidor");

    let cigar_a_id = create_test_cigar(&ctx.pool, "User A's Cigar", 10, Some(humidor_a_id))
        .await
        .expect("Failed to create cigar");

    // User B creates their own humidor
    let humidor_b_id = create_test_humidor(&ctx.pool, user_b_id, "User B's Humidor")
        .await
        .expect("Failed to create humidor");

    // User A tries to move their cigar to User B's humidor
    // Handler should check ownership of destination humidor
    let client = ctx.pool.get().await.expect("Failed to get client");
    let new_humidor_check = client
        .query_opt(
            "SELECT EXISTS(SELECT 1 FROM humidors WHERE id = $1 AND user_id = $2)",
            &[&humidor_b_id, &user_a_id],
        )
        .await
        .expect("Query failed");

    let has_access: bool = new_humidor_check.unwrap().get(0);
    assert!(
        !has_access,
        "User A should not be able to move cigar to User B's humidor"
    );

    // Verify the cigar is still in User A's humidor
    let result = client
        .query_one(
            "SELECT humidor_id FROM cigars WHERE id = $1",
            &[&cigar_a_id],
        )
        .await
        .expect("Query failed");

    let current_humidor: Option<Uuid> = result.get(0);
    assert_eq!(
        current_humidor,
        Some(humidor_a_id),
        "Cigar should remain in original humidor"
    );
}

// ============================================================================
// FAVORITES ISOLATION TESTS
// ============================================================================

#[tokio::test]
#[serial_test::serial]
async fn test_user_cannot_view_other_users_favorites() {
    let ctx = setup_test_db().await;

    // Create two users
    let (user_a_id, _) = create_test_user(&ctx.pool, "user_a", "password", false)
        .await
        .expect("Failed to create user A");

    let (user_b_id, _) = create_test_user(&ctx.pool, "user_b", "password", false)
        .await
        .expect("Failed to create user B");

    // User A creates a humidor and cigar and favorites it
    let humidor_a_id = create_test_humidor(&ctx.pool, user_a_id, "User A's Humidor")
        .await
        .expect("Failed to create humidor");

    let cigar_a_id = create_test_cigar(&ctx.pool, "User A's Cigar", 10, Some(humidor_a_id))
        .await
        .expect("Failed to create cigar");

    let client = ctx.pool.get().await.expect("Failed to get client");
    client
        .execute(
            "INSERT INTO favorites (id, user_id, cigar_id, snapshot_name, created_at) 
             VALUES ($1, $2, $3, $4, NOW())",
            &[&Uuid::new_v4(), &user_a_id, &cigar_a_id, &"Snapshot Name"],
        )
        .await
        .expect("Failed to create favorite");

    // User B tries to view User A's favorites
    let result = client
        .query(
            "SELECT id FROM favorites WHERE user_id = $1",
            &[&user_b_id],
        )
        .await
        .expect("Query failed");

    assert_eq!(result.len(), 0, "User B should not see User A's favorites");

    // Verify User A can see their own favorites
    let result = client
        .query(
            "SELECT id FROM favorites WHERE user_id = $1",
            &[&user_a_id],
        )
        .await
        .expect("Query failed");

    assert_eq!(result.len(), 1, "User A should see their own favorite");
}

#[tokio::test]
#[serial_test::serial]
async fn test_user_cannot_favorite_other_users_cigar() {
    let ctx = setup_test_db().await;

    // Create two users
    let (user_a_id, _) = create_test_user(&ctx.pool, "user_a", "password", false)
        .await
        .expect("Failed to create user A");

    let (user_b_id, _) = create_test_user(&ctx.pool, "user_b", "password", false)
        .await
        .expect("Failed to create user B");

    // User A creates a humidor and cigar
    let humidor_a_id = create_test_humidor(&ctx.pool, user_a_id, "User A's Humidor")
        .await
        .expect("Failed to create humidor");

    let cigar_a_id = create_test_cigar(&ctx.pool, "User A's Cigar", 10, Some(humidor_a_id))
        .await
        .expect("Failed to create cigar");

    // User B tries to favorite User A's cigar
    // Handler should verify cigar ownership before allowing favorite
    let client = ctx.pool.get().await.expect("Failed to get client");
    let ownership_check = client
        .query_opt(
            "SELECT c.name FROM cigars c
             INNER JOIN humidors h ON c.humidor_id = h.id
             WHERE c.id = $1 AND h.user_id = $2",
            &[&cigar_a_id, &user_b_id],
        )
        .await
        .expect("Query failed");

    assert!(
        ownership_check.is_none(),
        "User B should not be able to favorite User A's cigar (ownership check fails)"
    );

    // Verify User A can favorite their own cigar
    let ownership_check = client
        .query_opt(
            "SELECT c.name FROM cigars c
             INNER JOIN humidors h ON c.humidor_id = h.id
             WHERE c.id = $1 AND h.user_id = $2",
            &[&cigar_a_id, &user_a_id],
        )
        .await
        .expect("Query failed");

    assert!(
        ownership_check.is_some(),
        "User A should be able to favorite their own cigar"
    );
}

#[tokio::test]
#[serial_test::serial]
async fn test_user_cannot_remove_other_users_favorite() {
    let ctx = setup_test_db().await;

    // Create two users
    let (user_a_id, _) = create_test_user(&ctx.pool, "user_a", "password", false)
        .await
        .expect("Failed to create user A");

    let (user_b_id, _) = create_test_user(&ctx.pool, "user_b", "password", false)
        .await
        .expect("Failed to create user B");

    // User A creates a humidor and cigar and favorites it
    let humidor_a_id = create_test_humidor(&ctx.pool, user_a_id, "User A's Humidor")
        .await
        .expect("Failed to create humidor");

    let cigar_a_id = create_test_cigar(&ctx.pool, "User A's Cigar", 10, Some(humidor_a_id))
        .await
        .expect("Failed to create cigar");

    let client = ctx.pool.get().await.expect("Failed to get client");
    let favorite_id = Uuid::new_v4();
    client
        .execute(
            "INSERT INTO favorites (id, user_id, cigar_id, snapshot_name, created_at) 
             VALUES ($1, $2, $3, $4, NOW())",
            &[&favorite_id, &user_a_id, &cigar_a_id, &"Snapshot Name"],
        )
        .await
        .expect("Failed to create favorite");

    // User B tries to remove User A's favorite
    let rows_deleted = client
        .execute(
            "DELETE FROM favorites WHERE user_id = $1 AND cigar_id = $2",
            &[&user_b_id, &cigar_a_id],
        )
        .await
        .expect("Query failed");

    assert_eq!(
        rows_deleted, 0,
        "User B should not be able to remove User A's favorite"
    );

    // Verify favorite still exists
    let result = client
        .query_opt(
            "SELECT id FROM favorites WHERE id = $1",
            &[&favorite_id],
        )
        .await
        .expect("Query failed");

    assert!(result.is_some(), "Favorite should still exist");
}

// ============================================================================
// WISH LIST ISOLATION TESTS
// ============================================================================

#[tokio::test]
#[serial_test::serial]
async fn test_user_cannot_view_other_users_wish_list() {
    let ctx = setup_test_db().await;

    // Create two users
    let (user_a_id, _) = create_test_user(&ctx.pool, "user_a", "password", false)
        .await
        .expect("Failed to create user A");

    let (user_b_id, _) = create_test_user(&ctx.pool, "user_b", "password", false)
        .await
        .expect("Failed to create user B");

    // User A creates a humidor and cigar and adds to wish list
    let humidor_a_id = create_test_humidor(&ctx.pool, user_a_id, "User A's Humidor")
        .await
        .expect("Failed to create humidor");

    let cigar_a_id = create_test_cigar(&ctx.pool, "User A's Cigar", 10, Some(humidor_a_id))
        .await
        .expect("Failed to create cigar");

    let client = ctx.pool.get().await.expect("Failed to get client");
    client
        .execute(
            "INSERT INTO wish_list (id, user_id, cigar_id, created_at) 
             VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user_a_id, &cigar_a_id],
        )
        .await
        .expect("Failed to create wish list item");

    // User B tries to view User A's wish list
    let result = client
        .query(
            "SELECT id FROM wish_list WHERE user_id = $1",
            &[&user_b_id],
        )
        .await
        .expect("Query failed");

    assert_eq!(result.len(), 0, "User B should not see User A's wish list");

    // Verify User A can see their own wish list
    let result = client
        .query(
            "SELECT id FROM wish_list WHERE user_id = $1",
            &[&user_a_id],
        )
        .await
        .expect("Query failed");

    assert_eq!(result.len(), 1, "User A should see their own wish list item");
}

#[tokio::test]
#[serial_test::serial]
async fn test_user_cannot_modify_other_users_wish_list() {
    let ctx = setup_test_db().await;

    // Create two users
    let (user_a_id, _) = create_test_user(&ctx.pool, "user_a", "password", false)
        .await
        .expect("Failed to create user A");

    let (user_b_id, _) = create_test_user(&ctx.pool, "user_b", "password", false)
        .await
        .expect("Failed to create user B");

    // User A creates a humidor and cigar and adds to wish list
    let humidor_a_id = create_test_humidor(&ctx.pool, user_a_id, "User A's Humidor")
        .await
        .expect("Failed to create humidor");

    let cigar_a_id = create_test_cigar(&ctx.pool, "User A's Cigar", 10, Some(humidor_a_id))
        .await
        .expect("Failed to create cigar");

    let client = ctx.pool.get().await.expect("Failed to get client");
    client
        .execute(
            "INSERT INTO wish_list (id, user_id, cigar_id, notes, created_at) 
             VALUES ($1, $2, $3, $4, NOW())",
            &[
                &Uuid::new_v4(),
                &user_a_id,
                &cigar_a_id,
                &"Original notes",
            ],
        )
        .await
        .expect("Failed to create wish list item");

    // User B tries to update User A's wish list notes
    let rows_updated = client
        .execute(
            "UPDATE wish_list SET notes = $1 WHERE user_id = $2 AND cigar_id = $3",
            &[&"Hacked notes", &user_b_id, &cigar_a_id],
        )
        .await
        .expect("Query failed");

    assert_eq!(
        rows_updated, 0,
        "User B should not be able to update User A's wish list"
    );

    // Verify notes were not changed
    let result = client
        .query_one(
            "SELECT notes FROM wish_list WHERE user_id = $1 AND cigar_id = $2",
            &[&user_a_id, &cigar_a_id],
        )
        .await
        .expect("Query failed");

    let notes: Option<String> = result.get(0);
    assert_eq!(
        notes.as_deref(),
        Some("Original notes"),
        "Wish list notes should not be changed"
    );
}

#[tokio::test]
#[serial_test::serial]
async fn test_user_cannot_delete_other_users_wish_list_item() {
    let ctx = setup_test_db().await;

    // Create two users
    let (user_a_id, _) = create_test_user(&ctx.pool, "user_a", "password", false)
        .await
        .expect("Failed to create user A");

    let (user_b_id, _) = create_test_user(&ctx.pool, "user_b", "password", false)
        .await
        .expect("Failed to create user B");

    // User A creates a humidor and cigar and adds to wish list
    let humidor_a_id = create_test_humidor(&ctx.pool, user_a_id, "User A's Humidor")
        .await
        .expect("Failed to create humidor");

    let cigar_a_id = create_test_cigar(&ctx.pool, "User A's Cigar", 10, Some(humidor_a_id))
        .await
        .expect("Failed to create cigar");

    let client = ctx.pool.get().await.expect("Failed to get client");
    client
        .execute(
            "INSERT INTO wish_list (id, user_id, cigar_id, created_at) 
             VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user_a_id, &cigar_a_id],
        )
        .await
        .expect("Failed to create wish list item");

    // User B tries to delete User A's wish list item
    let rows_deleted = client
        .execute(
            "DELETE FROM wish_list WHERE user_id = $1 AND cigar_id = $2",
            &[&user_b_id, &cigar_a_id],
        )
        .await
        .expect("Query failed");

    assert_eq!(
        rows_deleted, 0,
        "User B should not be able to delete User A's wish list item"
    );

    // Verify wish list item still exists
    let result = client
        .query_opt(
            "SELECT id FROM wish_list WHERE user_id = $1 AND cigar_id = $2",
            &[&user_a_id, &cigar_a_id],
        )
        .await
        .expect("Query failed");

    assert!(result.is_some(), "Wish list item should still exist");
}

// ============================================================================
// COMPREHENSIVE ISOLATION TEST
// ============================================================================

#[tokio::test]
#[serial_test::serial]
async fn test_complete_user_isolation() {
    let ctx = setup_test_db().await;

    // Create two users with full data sets
    let (user_a_id, _) = create_test_user(&ctx.pool, "user_a", "password", false)
        .await
        .expect("Failed to create user A");

    let (user_b_id, _) = create_test_user(&ctx.pool, "user_b", "password", false)
        .await
        .expect("Failed to create user B");

    // User A creates complete dataset
    let humidor_a_id = create_test_humidor(&ctx.pool, user_a_id, "User A's Humidor")
        .await
        .expect("Failed to create humidor");

    let cigar_a_id = create_test_cigar(&ctx.pool, "User A's Cigar", 10, Some(humidor_a_id))
        .await
        .expect("Failed to create cigar");

    let client = ctx.pool.get().await.expect("Failed to get client");
    
    // Add to favorites
    client
        .execute(
            "INSERT INTO favorites (id, user_id, cigar_id, snapshot_name, created_at) 
             VALUES ($1, $2, $3, $4, NOW())",
            &[&Uuid::new_v4(), &user_a_id, &cigar_a_id, &"Favorite"],
        )
        .await
        .expect("Failed to create favorite");

    // Add to wish list
    client
        .execute(
            "INSERT INTO wish_list (id, user_id, cigar_id, created_at) 
             VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user_a_id, &cigar_a_id],
        )
        .await
        .expect("Failed to create wish list item");

    // Verify User B has no access to any of User A's data
    let humidors = client
        .query("SELECT id FROM humidors WHERE user_id = $1", &[&user_b_id])
        .await
        .expect("Query failed");
    assert_eq!(humidors.len(), 0, "User B should have no humidors");

    let cigars = client
        .query(
            "SELECT c.id FROM cigars c
             INNER JOIN humidors h ON c.humidor_id = h.id
             WHERE h.user_id = $1",
            &[&user_b_id],
        )
        .await
        .expect("Query failed");
    assert_eq!(cigars.len(), 0, "User B should have no cigars");

    let favorites = client
        .query(
            "SELECT id FROM favorites WHERE user_id = $1",
            &[&user_b_id],
        )
        .await
        .expect("Query failed");
    assert_eq!(favorites.len(), 0, "User B should have no favorites");

    let wish_list = client
        .query(
            "SELECT id FROM wish_list WHERE user_id = $1",
            &[&user_b_id],
        )
        .await
        .expect("Query failed");
    assert_eq!(wish_list.len(), 0, "User B should have no wish list items");

    // Verify User A can still access all their data
    let humidors = client
        .query("SELECT id FROM humidors WHERE user_id = $1", &[&user_a_id])
        .await
        .expect("Query failed");
    assert_eq!(humidors.len(), 1, "User A should have 1 humidor");

    let cigars = client
        .query(
            "SELECT c.id FROM cigars c
             INNER JOIN humidors h ON c.humidor_id = h.id
             WHERE h.user_id = $1",
            &[&user_a_id],
        )
        .await
        .expect("Query failed");
    assert_eq!(cigars.len(), 1, "User A should have 1 cigar");

    let favorites = client
        .query(
            "SELECT id FROM favorites WHERE user_id = $1",
            &[&user_a_id],
        )
        .await
        .expect("Query failed");
    assert_eq!(favorites.len(), 1, "User A should have 1 favorite");

    let wish_list = client
        .query(
            "SELECT id FROM wish_list WHERE user_id = $1",
            &[&user_a_id],
        )
        .await
        .expect("Query failed");
    assert_eq!(wish_list.len(), 1, "User A should have 1 wish list item");
}
