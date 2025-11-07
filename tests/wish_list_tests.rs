mod common;

use common::*;
use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_add_to_wish_list() {
    let ctx = setup_test_db().await;

    // Create user and cigar
    let (user_id, _username) = create_test_user(&ctx.pool, "testuser", "password123", false)
        .await
        .unwrap();
    let cigar_id = create_test_cigar(&ctx.pool, "Dream Cigar", 5, None)
        .await
        .unwrap();

    // Add to wish list
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one(
            "INSERT INTO wish_list (id, user_id, cigar_id, created_at) 
             VALUES ($1, $2, $3, NOW()) 
             RETURNING id",
            &[&Uuid::new_v4(), &user_id, &cigar_id],
        )
        .await
        .unwrap();

    let wish_list_id: Uuid = row.get(0);
    assert!(!wish_list_id.to_string().is_empty());
}

#[tokio::test]
#[serial]
async fn test_add_to_wish_list_with_notes() {
    let ctx = setup_test_db().await;

    // Create user and cigar
    let (user_id, _username) = create_test_user(&ctx.pool, "testuser", "password123", false)
        .await
        .unwrap();
    let cigar_id = create_test_cigar(&ctx.pool, "Rare Cigar", 5, None)
        .await
        .unwrap();

    // Add to wish list with notes
    let client = ctx.pool.get().await.unwrap();
    let notes = "Want to try this at the next cigar event";
    let row = client
        .query_one(
            "INSERT INTO wish_list (id, user_id, cigar_id, notes, created_at) 
             VALUES ($1, $2, $3, $4, NOW()) 
             RETURNING id, notes",
            &[&Uuid::new_v4(), &user_id, &cigar_id, &notes],
        )
        .await
        .unwrap();

    let stored_notes: Option<String> = row.get(1);
    assert_eq!(stored_notes, Some(notes.to_string()));
}

#[tokio::test]
#[serial]
async fn test_get_user_wish_list() {
    let ctx = setup_test_db().await;

    // Create user and multiple cigars
    let (user_id, _username) = create_test_user(&ctx.pool, "testuser", "password123", false)
        .await
        .unwrap();
    let cigar1_id = create_test_cigar(&ctx.pool, "Cigar 1", 5, None)
        .await
        .unwrap();
    let cigar2_id = create_test_cigar(&ctx.pool, "Cigar 2", 3, None)
        .await
        .unwrap();
    let _cigar3_id = create_test_cigar(&ctx.pool, "Cigar 3", 7, None)
        .await
        .unwrap();

    // Add to wish list
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "INSERT INTO wish_list (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user_id, &cigar1_id],
        )
        .await
        .unwrap();
    client
        .execute(
            "INSERT INTO wish_list (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user_id, &cigar2_id],
        )
        .await
        .unwrap();

    // Get wish list count
    let rows = client
        .query(
            "SELECT COUNT(*) FROM wish_list WHERE user_id = $1",
            &[&user_id],
        )
        .await
        .unwrap();

    let count: i64 = rows[0].get(0);
    assert_eq!(count, 2);
}

#[tokio::test]
#[serial]
async fn test_remove_from_wish_list() {
    let ctx = setup_test_db().await;

    // Create user and cigar
    let (user_id, _username) = create_test_user(&ctx.pool, "testuser", "password123", false)
        .await
        .unwrap();
    let cigar_id = create_test_cigar(&ctx.pool, "Wish List Cigar", 5, None)
        .await
        .unwrap();

    // Add to wish list
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one(
            "INSERT INTO wish_list (id, user_id, cigar_id, created_at) 
             VALUES ($1, $2, $3, NOW()) 
             RETURNING id",
            &[&Uuid::new_v4(), &user_id, &cigar_id],
        )
        .await
        .unwrap();

    let wish_list_id: Uuid = row.get(0);

    // Remove from wish list
    let rows_affected = client
        .execute("DELETE FROM wish_list WHERE id = $1", &[&wish_list_id])
        .await
        .unwrap();

    assert_eq!(rows_affected, 1);

    // Verify it's removed
    let result = client
        .query_opt("SELECT id FROM wish_list WHERE id = $1", &[&wish_list_id])
        .await
        .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
#[serial]
async fn test_duplicate_wish_list_prevention() {
    let ctx = setup_test_db().await;

    // Create user and cigar
    let (user_id, _username) = create_test_user(&ctx.pool, "testuser", "password123", false)
        .await
        .unwrap();
    let cigar_id = create_test_cigar(&ctx.pool, "Wish List Cigar", 5, None)
        .await
        .unwrap();

    // Add to wish list
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "INSERT INTO wish_list (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user_id, &cigar_id],
        )
        .await
        .unwrap();

    // Try to add same cigar again
    let result = client
        .execute(
            "INSERT INTO wish_list (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user_id, &cigar_id],
        )
        .await;

    // Should fail due to unique constraint on (user_id, cigar_id)
    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn test_check_if_cigar_is_in_wish_list() {
    let ctx = setup_test_db().await;

    // Create user and cigars
    let (user_id, _username) = create_test_user(&ctx.pool, "testuser", "password123", false)
        .await
        .unwrap();
    let wishlisted_cigar_id = create_test_cigar(&ctx.pool, "On Wish List", 5, None)
        .await
        .unwrap();
    let not_wishlisted_cigar_id = create_test_cigar(&ctx.pool, "Not on Wish List", 3, None)
        .await
        .unwrap();

    // Add one to wish list
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "INSERT INTO wish_list (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user_id, &wishlisted_cigar_id],
        )
        .await
        .unwrap();

    // Check if in wish list
    let result = client
        .query_opt(
            "SELECT id FROM wish_list WHERE user_id = $1 AND cigar_id = $2",
            &[&user_id, &wishlisted_cigar_id],
        )
        .await
        .unwrap();
    assert!(result.is_some());

    // Check if not in wish list
    let result = client
        .query_opt(
            "SELECT id FROM wish_list WHERE user_id = $1 AND cigar_id = $2",
            &[&user_id, &not_wishlisted_cigar_id],
        )
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
#[serial]
async fn test_wish_list_separated_by_user() {
    let ctx = setup_test_db().await;

    // Create two users
    let (user1_id, _username1) = create_test_user(&ctx.pool, "user1", "password1", false)
        .await
        .unwrap();
    let (user2_id, _username2) = create_test_user(&ctx.pool, "user2", "password2", false)
        .await
        .unwrap();

    // Create cigar
    let cigar_id = create_test_cigar(&ctx.pool, "Popular Cigar", 10, None)
        .await
        .unwrap();

    // Both users add to wish list
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "INSERT INTO wish_list (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user1_id, &cigar_id],
        )
        .await
        .unwrap();
    client
        .execute(
            "INSERT INTO wish_list (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user2_id, &cigar_id],
        )
        .await
        .unwrap();

    // Each user should see their own wish list
    let rows = client
        .query(
            "SELECT COUNT(*) FROM wish_list WHERE user_id = $1",
            &[&user1_id],
        )
        .await
        .unwrap();
    let user1_count: i64 = rows[0].get(0);

    let rows = client
        .query(
            "SELECT COUNT(*) FROM wish_list WHERE user_id = $1",
            &[&user2_id],
        )
        .await
        .unwrap();
    let user2_count: i64 = rows[0].get(0);

    assert_eq!(user1_count, 1);
    assert_eq!(user2_count, 1);
}

#[tokio::test]
#[serial]
async fn test_get_wish_list_with_cigar_details() {
    let ctx = setup_test_db().await;

    // Create user and cigar
    let (user_id, _username) = create_test_user(&ctx.pool, "testuser", "password123", false)
        .await
        .unwrap();
    let cigar_id = create_test_cigar(&ctx.pool, "Dream Cigar", 5, None)
        .await
        .unwrap();

    // Add to wish list
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "INSERT INTO wish_list (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user_id, &cigar_id],
        )
        .await
        .unwrap();

    // Get wish list with cigar details (JOIN)
    let rows = client
        .query(
            "SELECT c.id, c.name, c.quantity 
             FROM wish_list w 
             JOIN cigars c ON w.cigar_id = c.id 
             WHERE w.user_id = $1",
            &[&user_id],
        )
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);

    let cigar_name: String = rows[0].get(1);
    let quantity: i32 = rows[0].get(2);

    assert_eq!(cigar_name, "Dream Cigar");
    assert_eq!(quantity, 5);
}

#[tokio::test]
#[serial]
async fn test_update_wish_list_notes() {
    let ctx = setup_test_db().await;

    // Create user and cigar
    let (user_id, _username) = create_test_user(&ctx.pool, "testuser", "password123", false)
        .await
        .unwrap();
    let cigar_id = create_test_cigar(&ctx.pool, "Rare Cigar", 5, None)
        .await
        .unwrap();

    // Add to wish list with initial notes
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one(
            "INSERT INTO wish_list (id, user_id, cigar_id, notes, created_at) 
             VALUES ($1, $2, $3, $4, NOW()) 
             RETURNING id",
            &[&Uuid::new_v4(), &user_id, &cigar_id, &"Initial notes"],
        )
        .await
        .unwrap();

    let wish_list_id: Uuid = row.get(0);

    // Update notes
    client
        .execute(
            "UPDATE wish_list SET notes = $1 WHERE id = $2",
            &[&"Updated notes - saw this at the shop", &wish_list_id],
        )
        .await
        .unwrap();

    // Verify update
    let row = client
        .query_one(
            "SELECT notes FROM wish_list WHERE id = $1",
            &[&wish_list_id],
        )
        .await
        .unwrap();

    let notes: Option<String> = row.get(0);
    assert_eq!(
        notes,
        Some("Updated notes - saw this at the shop".to_string())
    );
}

#[tokio::test]
#[serial]
async fn test_cigar_can_be_in_both_favorites_and_wish_list() {
    let ctx = setup_test_db().await;

    // Create user and cigar
    let (user_id, _username) = create_test_user(&ctx.pool, "testuser", "password123", false)
        .await
        .unwrap();
    let cigar_id = create_test_cigar(&ctx.pool, "Amazing Cigar", 5, None)
        .await
        .unwrap();

    let client = ctx.pool.get().await.unwrap();

    // Add to favorites
    client
        .execute(
            "INSERT INTO favorites (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user_id, &cigar_id],
        )
        .await
        .unwrap();

    // Add to wish list
    client
        .execute(
            "INSERT INTO wish_list (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user_id, &cigar_id],
        )
        .await
        .unwrap();

    // Verify it's in both
    let favorites_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM favorites WHERE user_id = $1 AND cigar_id = $2",
            &[&user_id, &cigar_id],
        )
        .await
        .unwrap()
        .get(0);

    let wish_list_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM wish_list WHERE user_id = $1 AND cigar_id = $2",
            &[&user_id, &cigar_id],
        )
        .await
        .unwrap()
        .get(0);

    assert_eq!(favorites_count, 1);
    assert_eq!(wish_list_count, 1);
}
