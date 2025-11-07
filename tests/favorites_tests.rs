mod common;

use common::*;
use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_add_favorite() {
    let ctx = setup_test_db().await;

    // Create user and cigar
    let (user_id, _username) = create_test_user(&ctx.pool, "testuser", "password123", false)
        .await
        .unwrap();
    let cigar_id = create_test_cigar(&ctx.pool, "Favorite Cigar", 5, None)
        .await
        .unwrap();

    // Add to favorites
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one(
            "INSERT INTO favorites (id, user_id, cigar_id, created_at) 
             VALUES ($1, $2, $3, NOW()) 
             RETURNING id",
            &[&Uuid::new_v4(), &user_id, &cigar_id],
        )
        .await
        .unwrap();

    let favorite_id: Uuid = row.get(0);
    assert!(!favorite_id.to_string().is_empty());
}

#[tokio::test]
#[serial]
async fn test_get_user_favorites() {
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

    // Add to favorites
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "INSERT INTO favorites (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user_id, &cigar1_id],
        )
        .await
        .unwrap();
    client
        .execute(
            "INSERT INTO favorites (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user_id, &cigar2_id],
        )
        .await
        .unwrap();

    // Get favorites count
    let rows = client
        .query(
            "SELECT COUNT(*) FROM favorites WHERE user_id = $1",
            &[&user_id],
        )
        .await
        .unwrap();

    let count: i64 = rows[0].get(0);
    assert_eq!(count, 2);
}

#[tokio::test]
#[serial]
async fn test_remove_favorite() {
    let ctx = setup_test_db().await;

    // Create user and cigar
    let (user_id, _username) = create_test_user(&ctx.pool, "testuser", "password123", false)
        .await
        .unwrap();
    let cigar_id = create_test_cigar(&ctx.pool, "Favorite Cigar", 5, None)
        .await
        .unwrap();

    // Add to favorites
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one(
            "INSERT INTO favorites (id, user_id, cigar_id, created_at) 
             VALUES ($1, $2, $3, NOW()) 
             RETURNING id",
            &[&Uuid::new_v4(), &user_id, &cigar_id],
        )
        .await
        .unwrap();

    let favorite_id: Uuid = row.get(0);

    // Remove from favorites
    let rows_affected = client
        .execute("DELETE FROM favorites WHERE id = $1", &[&favorite_id])
        .await
        .unwrap();

    assert_eq!(rows_affected, 1);

    // Verify it's removed
    let result = client
        .query_opt("SELECT id FROM favorites WHERE id = $1", &[&favorite_id])
        .await
        .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
#[serial]
async fn test_duplicate_favorite_prevention() {
    let ctx = setup_test_db().await;

    // Create user and cigar
    let (user_id, _username) = create_test_user(&ctx.pool, "testuser", "password123", false)
        .await
        .unwrap();
    let cigar_id = create_test_cigar(&ctx.pool, "Favorite Cigar", 5, None)
        .await
        .unwrap();

    // Add to favorites
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "INSERT INTO favorites (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user_id, &cigar_id],
        )
        .await
        .unwrap();

    // Try to add same cigar again
    let result = client
        .execute(
            "INSERT INTO favorites (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user_id, &cigar_id],
        )
        .await;

    // Should fail due to unique constraint on (user_id, cigar_id)
    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn test_check_if_cigar_is_favorited() {
    let ctx = setup_test_db().await;

    // Create user and cigar
    let (user_id, _username) = create_test_user(&ctx.pool, "testuser", "password123", false)
        .await
        .unwrap();
    let favorited_cigar_id = create_test_cigar(&ctx.pool, "Favorited", 5, None)
        .await
        .unwrap();
    let not_favorited_cigar_id = create_test_cigar(&ctx.pool, "Not Favorited", 3, None)
        .await
        .unwrap();

    // Add one to favorites
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "INSERT INTO favorites (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user_id, &favorited_cigar_id],
        )
        .await
        .unwrap();

    // Check if favorited
    let result = client
        .query_opt(
            "SELECT id FROM favorites WHERE user_id = $1 AND cigar_id = $2",
            &[&user_id, &favorited_cigar_id],
        )
        .await
        .unwrap();
    assert!(result.is_some());

    // Check if not favorited
    let result = client
        .query_opt(
            "SELECT id FROM favorites WHERE user_id = $1 AND cigar_id = $2",
            &[&user_id, &not_favorited_cigar_id],
        )
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
#[serial]
async fn test_favorites_separated_by_user() {
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

    // Both users favorite the same cigar
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "INSERT INTO favorites (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user1_id, &cigar_id],
        )
        .await
        .unwrap();
    client
        .execute(
            "INSERT INTO favorites (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user2_id, &cigar_id],
        )
        .await
        .unwrap();

    // Each user should see their own favorite
    let rows = client
        .query(
            "SELECT COUNT(*) FROM favorites WHERE user_id = $1",
            &[&user1_id],
        )
        .await
        .unwrap();
    let user1_count: i64 = rows[0].get(0);

    let rows = client
        .query(
            "SELECT COUNT(*) FROM favorites WHERE user_id = $1",
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
async fn test_get_favorites_with_cigar_details() {
    let ctx = setup_test_db().await;

    // Create user and cigars
    let (user_id, _username) = create_test_user(&ctx.pool, "testuser", "password123", false)
        .await
        .unwrap();
    let cigar_id = create_test_cigar(&ctx.pool, "Premium Cigar", 5, None)
        .await
        .unwrap();

    // Add to favorites
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "INSERT INTO favorites (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&Uuid::new_v4(), &user_id, &cigar_id],
        )
        .await
        .unwrap();

    // Get favorites with cigar details (JOIN)
    let rows = client
        .query(
            "SELECT c.id, c.name, c.quantity 
             FROM favorites f 
             JOIN cigars c ON f.cigar_id = c.id 
             WHERE f.user_id = $1",
            &[&user_id],
        )
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);

    let cigar_name: String = rows[0].get(1);
    let quantity: i32 = rows[0].get(2);

    assert_eq!(cigar_name, "Premium Cigar");
    assert_eq!(quantity, 5);
}
