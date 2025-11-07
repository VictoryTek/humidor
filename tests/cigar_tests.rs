mod common;

use common::*;
use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_create_cigar() {
    let ctx = setup_test_db().await;

    let cigar_id = create_test_cigar(&ctx.pool, "Test Cigar", 5, None)
        .await
        .expect("Failed to create cigar");

    // Verify cigar exists
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one(
            "SELECT name, quantity, is_active FROM cigars WHERE id = $1",
            &[&cigar_id],
        )
        .await
        .unwrap();

    let name: String = row.get(0);
    let quantity: i32 = row.get(1);
    let is_active: bool = row.get(2);

    assert_eq!(name, "Test Cigar");
    assert_eq!(quantity, 5);
    assert!(is_active);
}

#[tokio::test]
#[serial]
async fn test_create_cigar_with_humidor() {
    let ctx = setup_test_db().await;

    // Create user and humidor
    let (user_id, _username) = create_test_user(&ctx.pool, "testuser", "password123", false)
        .await
        .unwrap();
    let humidor_id = create_test_humidor(&ctx.pool, user_id, "My Humidor")
        .await
        .unwrap();

    // Create cigar in humidor
    let cigar_id = create_test_cigar(&ctx.pool, "Humidor Cigar", 10, Some(humidor_id))
        .await
        .unwrap();

    // Verify cigar is associated with humidor
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one("SELECT humidor_id FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();

    let stored_humidor_id: Option<Uuid> = row.get(0);
    assert_eq!(stored_humidor_id, Some(humidor_id));
}

#[tokio::test]
#[serial]
async fn test_read_cigar() {
    let ctx = setup_test_db().await;

    let cigar_id = create_test_cigar(&ctx.pool, "Read Test Cigar", 3, None)
        .await
        .unwrap();

    // Read cigar
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one(
            "SELECT id, name, quantity FROM cigars WHERE id = $1",
            &[&cigar_id],
        )
        .await
        .unwrap();

    let id: Uuid = row.get(0);
    let name: String = row.get(1);
    let quantity: i32 = row.get(2);

    assert_eq!(id, cigar_id);
    assert_eq!(name, "Read Test Cigar");
    assert_eq!(quantity, 3);
}

#[tokio::test]
#[serial]
async fn test_update_cigar_quantity() {
    let ctx = setup_test_db().await;

    let cigar_id = create_test_cigar(&ctx.pool, "Update Test", 5, None)
        .await
        .unwrap();

    // Update quantity
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "UPDATE cigars SET quantity = $1, updated_at = NOW() WHERE id = $2",
            &[&10, &cigar_id],
        )
        .await
        .unwrap();

    // Verify update
    let row = client
        .query_one("SELECT quantity FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();

    let quantity: i32 = row.get(0);
    assert_eq!(quantity, 10);
}

#[tokio::test]
#[serial]
async fn test_update_cigar_name() {
    let ctx = setup_test_db().await;

    let cigar_id = create_test_cigar(&ctx.pool, "Old Name", 5, None)
        .await
        .unwrap();

    // Update name
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "UPDATE cigars SET name = $1, updated_at = NOW() WHERE id = $2",
            &[&"New Name", &cigar_id],
        )
        .await
        .unwrap();

    // Verify update
    let row = client
        .query_one("SELECT name FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();

    let name: String = row.get(0);
    assert_eq!(name, "New Name");
}

#[tokio::test]
#[serial]
async fn test_delete_cigar() {
    let ctx = setup_test_db().await;

    let cigar_id = create_test_cigar(&ctx.pool, "Delete Test", 5, None)
        .await
        .unwrap();

    // Soft delete (set is_active = false)
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "UPDATE cigars SET is_active = false, updated_at = NOW() WHERE id = $1",
            &[&cigar_id],
        )
        .await
        .unwrap();

    // Verify cigar is marked inactive
    let row = client
        .query_one("SELECT is_active FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();

    let is_active: bool = row.get(0);
    assert!(!is_active);
}

#[tokio::test]
#[serial]
async fn test_hard_delete_cigar() {
    let ctx = setup_test_db().await;

    let cigar_id = create_test_cigar(&ctx.pool, "Hard Delete Test", 5, None)
        .await
        .unwrap();

    // Hard delete
    let client = ctx.pool.get().await.unwrap();
    let rows_affected = client
        .execute("DELETE FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();

    assert_eq!(rows_affected, 1);

    // Verify cigar no longer exists
    let result = client
        .query_opt("SELECT id FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
#[serial]
async fn test_list_all_cigars() {
    let ctx = setup_test_db().await;

    // Create multiple cigars
    create_test_cigar(&ctx.pool, "Cigar 1", 5, None)
        .await
        .unwrap();
    create_test_cigar(&ctx.pool, "Cigar 2", 3, None)
        .await
        .unwrap();
    create_test_cigar(&ctx.pool, "Cigar 3", 7, None)
        .await
        .unwrap();

    // List all cigars
    let client = ctx.pool.get().await.unwrap();
    let rows = client
        .query("SELECT COUNT(*) FROM cigars WHERE is_active = true", &[])
        .await
        .unwrap();

    let count: i64 = rows[0].get(0);
    assert_eq!(count, 3);
}

#[tokio::test]
#[serial]
async fn test_filter_cigars_by_humidor() {
    let ctx = setup_test_db().await;

    // Create user and humidors
    let (user_id, _username) = create_test_user(&ctx.pool, "testuser", "password123", false)
        .await
        .unwrap();
    let humidor1_id = create_test_humidor(&ctx.pool, user_id, "Humidor 1")
        .await
        .unwrap();
    let humidor2_id = create_test_humidor(&ctx.pool, user_id, "Humidor 2")
        .await
        .unwrap();

    // Create cigars in different humidors
    create_test_cigar(&ctx.pool, "Cigar A", 5, Some(humidor1_id))
        .await
        .unwrap();
    create_test_cigar(&ctx.pool, "Cigar B", 3, Some(humidor1_id))
        .await
        .unwrap();
    create_test_cigar(&ctx.pool, "Cigar C", 7, Some(humidor2_id))
        .await
        .unwrap();

    // Filter by humidor1
    let client = ctx.pool.get().await.unwrap();
    let rows = client
        .query(
            "SELECT COUNT(*) FROM cigars WHERE humidor_id = $1 AND is_active = true",
            &[&humidor1_id],
        )
        .await
        .unwrap();

    let count: i64 = rows[0].get(0);
    assert_eq!(count, 2);
}

#[tokio::test]
#[serial]
async fn test_cigar_quantity_zero_is_valid() {
    let ctx = setup_test_db().await;

    let cigar_id = create_test_cigar(&ctx.pool, "Empty Stock", 0, None)
        .await
        .expect("Should allow zero quantity");

    // Verify quantity is 0
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one("SELECT quantity FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();

    let quantity: i32 = row.get(0);
    assert_eq!(quantity, 0);
}
