mod common;

use common::*;
use serial_test::serial;

/// Test humidor CRUD operations
#[tokio::test]
#[serial]
async fn test_humidor_crud() {
    let ctx = setup_test_db().await;
    
    // Create user
    let (user_id, _username) = create_test_user(&ctx.pool, "humidor_user", "password", false)
        .await
        .unwrap();
    
    // Create humidor
    let humidor_id = create_test_humidor(&ctx.pool, user_id, "Test Humidor")
        .await
        .unwrap();
    
    // Verify humidor exists
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one(
            "SELECT name, user_id FROM humidors WHERE id = $1",
            &[&humidor_id],
        )
        .await
        .unwrap();
    
    let name: String = row.get(0);
    let owner_id: uuid::Uuid = row.get(1);
    
    assert_eq!(name, "Test Humidor");
    assert_eq!(owner_id, user_id);
}

/// Test that humidors belong to specific users
#[tokio::test]
#[serial]
async fn test_humidor_user_isolation() {
    let ctx = setup_test_db().await;
    
    // Create two users
    let (user1_id, _) = create_test_user(&ctx.pool, "user1", "password", false).await.unwrap();
    let (user2_id, _) = create_test_user(&ctx.pool, "user2", "password", false).await.unwrap();
    
    // Each creates a humidor
    let humidor1_id = create_test_humidor(&ctx.pool, user1_id, "User 1 Humidor").await.unwrap();
    let humidor2_id = create_test_humidor(&ctx.pool, user2_id, "User 2 Humidor").await.unwrap();
    
    // Verify each humidor belongs to correct user
    let client = ctx.pool.get().await.unwrap();
    
    let row1 = client
        .query_one("SELECT user_id FROM humidors WHERE id = $1", &[&humidor1_id])
        .await
        .unwrap();
    let owner1: uuid::Uuid = row1.get(0);
    assert_eq!(owner1, user1_id);
    
    let row2 = client
        .query_one("SELECT user_id FROM humidors WHERE id = $1", &[&humidor2_id])
        .await
        .unwrap();
    let owner2: uuid::Uuid = row2.get(0);
    assert_eq!(owner2, user2_id);
}

/// Test cigar quantity tracking
#[tokio::test]
#[serial]
async fn test_cigar_quantity_tracking() {
    let ctx = setup_test_db().await;
    
    // Create cigar with quantity
    let cigar_id = create_test_cigar(&ctx.pool, "Test Cigar", 10, None)
        .await
        .unwrap();
    
    // Verify quantity
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one("SELECT quantity FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();
    
    let quantity: i32 = row.get(0);
    assert_eq!(quantity, 10);
    
    // Update quantity
    client
        .execute("UPDATE cigars SET quantity = $1 WHERE id = $2", &[&7, &cigar_id])
        .await
        .unwrap();
    
    // Verify update
    let row = client
        .query_one("SELECT quantity FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();
    
    let new_quantity: i32 = row.get(0);
    assert_eq!(new_quantity, 7);
}

/// Test cigar belongs to humidor
#[tokio::test]
#[serial]
async fn test_cigar_humidor_relationship() {
    let ctx = setup_test_db().await;
    
    let (user_id, _) = create_test_user(&ctx.pool, "cigar_user", "password", false).await.unwrap();
    let humidor_id = create_test_humidor(&ctx.pool, user_id, "Cigar Humidor").await.unwrap();
    
    // Create cigar in humidor
    let cigar_id = create_test_cigar(&ctx.pool, "Humidor Cigar", 5, Some(humidor_id))
        .await
        .unwrap();
    
    // Verify relationship
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one("SELECT humidor_id FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();
    
    let cigar_humidor: Option<uuid::Uuid> = row.get(0);
    assert_eq!(cigar_humidor, Some(humidor_id));
}

/// Test favorite persistence
#[tokio::test]
#[serial]
async fn test_favorite_persistence() {
    let ctx = setup_test_db().await;
    
    let (user_id, _) = create_test_user(&ctx.pool, "fav_user", "password", false).await.unwrap();
    let cigar_id = create_test_cigar(&ctx.pool, "Favorite Cigar", 3, None).await.unwrap();
    
    // Add to favorites
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "INSERT INTO favorites (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&uuid::Uuid::new_v4(), &user_id, &cigar_id],
        )
        .await
        .unwrap();
    
    // Verify favorite exists
    let rows = client
        .query(
            "SELECT cigar_id FROM favorites WHERE user_id = $1",
            &[&user_id],
        )
        .await
        .unwrap();
    
    assert_eq!(rows.len(), 1);
    let fav_cigar_id: uuid::Uuid = rows[0].get(0);
    assert_eq!(fav_cigar_id, cigar_id);
}

/// Test wish list with notes
#[tokio::test]
#[serial]
async fn test_wish_list_with_notes() {
    let ctx = setup_test_db().await;
    
    let (user_id, _) = create_test_user(&ctx.pool, "wish_user", "password", false).await.unwrap();
    let cigar_id = create_test_cigar(&ctx.pool, "Wish Cigar", 0, None).await.unwrap();
    
    let notes = "Want to try this at the next event";
    
    // Add to wish list
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "INSERT INTO wish_list (id, user_id, cigar_id, notes, created_at) VALUES ($1, $2, $3, $4, NOW())",
            &[&uuid::Uuid::new_v4(), &user_id, &cigar_id, &notes],
        )
        .await
        .unwrap();
    
    // Verify wish list entry
    let row = client
        .query_one(
            "SELECT notes FROM wish_list WHERE user_id = $1 AND cigar_id = $2",
            &[&user_id, &cigar_id],
        )
        .await
        .unwrap();
    
    let saved_notes: Option<String> = row.get(0);
    assert_eq!(saved_notes, Some(notes.to_string()));
}

/// Test unique constraint on favorites
#[tokio::test]
#[serial]
async fn test_favorite_unique_constraint() {
    let ctx = setup_test_db().await;
    
    let (user_id, _) = create_test_user(&ctx.pool, "unique_user", "password", false).await.unwrap();
    let cigar_id = create_test_cigar(&ctx.pool, "Unique Cigar", 5, None).await.unwrap();
    
    let client = ctx.pool.get().await.unwrap();
    
    // Add to favorites first time - should succeed
    let result1 = client
        .execute(
            "INSERT INTO favorites (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&uuid::Uuid::new_v4(), &user_id, &cigar_id],
        )
        .await;
    assert!(result1.is_ok());
    
    // Try to add same cigar again - should fail due to unique constraint
    let result2 = client
        .execute(
            "INSERT INTO favorites (id, user_id, cigar_id, created_at) VALUES ($1, $2, $3, NOW())",
            &[&uuid::Uuid::new_v4(), &user_id, &cigar_id],
        )
        .await;
    assert!(result2.is_err());
}

/// Test cascade delete when user is deleted
#[tokio::test]
#[serial]
async fn test_cascade_delete_user_humidors() {
    let ctx = setup_test_db().await;
    
    let (user_id, _) = create_test_user(&ctx.pool, "cascade_user", "password", false).await.unwrap();
    let _humidor_id = create_test_humidor(&ctx.pool, user_id, "Will Be Deleted").await.unwrap();
    
    let client = ctx.pool.get().await.unwrap();
    
    // Verify humidor exists
    let count_before: i64 = client
        .query_one("SELECT COUNT(*) FROM humidors WHERE user_id = $1", &[&user_id])
        .await
        .unwrap()
        .get(0);
    assert_eq!(count_before, 1);
    
    // Delete user
    client
        .execute("DELETE FROM users WHERE id = $1", &[&user_id])
        .await
        .unwrap();
    
    // Verify humidors are cascade deleted
    let count_after: i64 = client
        .query_one("SELECT COUNT(*) FROM humidors WHERE user_id = $1", &[&user_id])
        .await
        .unwrap()
        .get(0);
    assert_eq!(count_after, 0);
}

/// Test admin flag on users
#[tokio::test]
#[serial]
async fn test_user_admin_flag() {
    let ctx = setup_test_db().await;
    
    let (admin_id, _) = create_test_user(&ctx.pool, "admin", "password", true).await.unwrap();
    let (user_id, _) = create_test_user(&ctx.pool, "regular", "password", false).await.unwrap();
    
    let client = ctx.pool.get().await.unwrap();
    
    let admin_row = client
        .query_one("SELECT is_admin FROM users WHERE id = $1", &[&admin_id])
        .await
        .unwrap();
    let is_admin: bool = admin_row.get(0);
    assert!(is_admin);
    
    let user_row = client
        .query_one("SELECT is_admin FROM users WHERE id = $1", &[&user_id])
        .await
        .unwrap();
    let is_regular: bool = user_row.get(0);
    assert!(!is_regular);
}

/// Test concurrent database access
#[tokio::test]
#[serial]
async fn test_concurrent_database_operations() {
    let ctx = setup_test_db().await;
    
    // Spawn multiple tasks that access database concurrently
    let mut handles = vec![];
    
    for i in 0..5 {
        let pool = ctx.pool.clone();
        let handle = tokio::spawn(async move {
            let client = pool.get().await.unwrap();
            let row = client
                .query_one("SELECT $1::int as num", &[&i])
                .await
                .unwrap();
            let num: i32 = row.get(0);
            assert_eq!(num, i);
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
}
