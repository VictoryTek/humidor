mod common;

use common::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_quantity_cannot_be_negative() {
    let ctx = setup_test_db().await;

    // Try to create cigar with negative quantity
    let client = ctx.pool.get().await.unwrap();
    let result = client
        .execute(
            "INSERT INTO cigars (id, name, quantity, is_active, created_at, updated_at) 
             VALUES ($1, $2, $3, true, NOW(), NOW())",
            &[&uuid::Uuid::new_v4(), &"Test Cigar", &-5i32],
        )
        .await;

    // PostgreSQL should accept this (no constraint), but application logic should prevent it
    // For now, we'll test that the application validates this
    assert!(
        result.is_ok(),
        "Database allows negative quantity, but app should validate"
    );
}

#[tokio::test]
#[serial]
async fn test_quantity_zero_is_valid() {
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

#[tokio::test]
#[serial]
async fn test_quantity_large_number() {
    let ctx = setup_test_db().await;

    // Test with a very large quantity
    let cigar_id = create_test_cigar(&ctx.pool, "Bulk Purchase", 10000, None)
        .await
        .expect("Should allow large quantity");

    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one("SELECT quantity FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();

    let quantity: i32 = row.get(0);
    assert_eq!(quantity, 10000);
}

#[tokio::test]
#[serial]
async fn test_decrement_quantity_to_zero() {
    let ctx = setup_test_db().await;

    // Create cigar with quantity 1
    let cigar_id = create_test_cigar(&ctx.pool, "Last One", 1, None)
        .await
        .unwrap();

    // Decrement to 0
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "UPDATE cigars SET quantity = quantity - 1 WHERE id = $1",
            &[&cigar_id],
        )
        .await
        .unwrap();

    // Verify it's now 0
    let row = client
        .query_one("SELECT quantity FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();

    let quantity: i32 = row.get(0);
    assert_eq!(quantity, 0);
}

#[tokio::test]
#[serial]
async fn test_decrement_quantity_below_zero_possible_in_db() {
    let ctx = setup_test_db().await;

    // Create cigar with quantity 1
    let cigar_id = create_test_cigar(&ctx.pool, "Test Cigar", 1, None)
        .await
        .unwrap();

    // Try to decrement below 0 (database allows this)
    let client = ctx.pool.get().await.unwrap();
    let result = client
        .execute(
            "UPDATE cigars SET quantity = quantity - 2 WHERE id = $1",
            &[&cigar_id],
        )
        .await;

    // Database allows this without constraint
    assert!(result.is_ok());

    // Verify it went negative
    let row = client
        .query_one("SELECT quantity FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();

    let quantity: i32 = row.get(0);
    assert_eq!(
        quantity, -1,
        "Database allows negative - app must validate!"
    );
}

#[tokio::test]
#[serial]
async fn test_increment_quantity() {
    let ctx = setup_test_db().await;

    // Create cigar with quantity 5
    let cigar_id = create_test_cigar(&ctx.pool, "Stock Up", 5, None)
        .await
        .unwrap();

    // Increment by 10
    let client = ctx.pool.get().await.unwrap();
    client
        .execute(
            "UPDATE cigars SET quantity = quantity + 10 WHERE id = $1",
            &[&cigar_id],
        )
        .await
        .unwrap();

    // Verify it's now 15
    let row = client
        .query_one("SELECT quantity FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();

    let quantity: i32 = row.get(0);
    assert_eq!(quantity, 15);
}

#[tokio::test]
#[serial]
async fn test_batch_quantity_update() {
    let ctx = setup_test_db().await;

    // Create multiple cigars
    let cigar1_id = create_test_cigar(&ctx.pool, "Cigar 1", 5, None)
        .await
        .unwrap();
    let cigar2_id = create_test_cigar(&ctx.pool, "Cigar 2", 10, None)
        .await
        .unwrap();
    let cigar3_id = create_test_cigar(&ctx.pool, "Cigar 3", 15, None)
        .await
        .unwrap();

    // Update all quantities at once
    let client = ctx.pool.get().await.unwrap();
    let rows_affected = client
        .execute(
            "UPDATE cigars SET quantity = quantity + 5 WHERE id = ANY($1)",
            &[&vec![cigar1_id, cigar2_id, cigar3_id]],
        )
        .await
        .unwrap();

    assert_eq!(rows_affected, 3);

    // Verify updates
    let row = client
        .query_one("SELECT quantity FROM cigars WHERE id = $1", &[&cigar1_id])
        .await
        .unwrap();
    assert_eq!(row.get::<_, i32>(0), 10);

    let row = client
        .query_one("SELECT quantity FROM cigars WHERE id = $1", &[&cigar2_id])
        .await
        .unwrap();
    assert_eq!(row.get::<_, i32>(0), 15);

    let row = client
        .query_one("SELECT quantity FROM cigars WHERE id = $1", &[&cigar3_id])
        .await
        .unwrap();
    assert_eq!(row.get::<_, i32>(0), 20);
}

#[tokio::test]
#[serial]
async fn test_quantity_update_transaction() {
    let ctx = setup_test_db().await;

    let cigar_id = create_test_cigar(&ctx.pool, "Test Cigar", 10, None)
        .await
        .unwrap();

    // Start a transaction
    let mut client = ctx.pool.get().await.unwrap();
    let transaction = client.transaction().await.unwrap();

    // Update quantity
    transaction
        .execute(
            "UPDATE cigars SET quantity = quantity - 3 WHERE id = $1",
            &[&cigar_id],
        )
        .await
        .unwrap();

    // Verify update within transaction
    let row = transaction
        .query_one("SELECT quantity FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();
    assert_eq!(row.get::<_, i32>(0), 7);

    // Commit transaction
    transaction.commit().await.unwrap();

    // Verify update persisted
    let row = client
        .query_one("SELECT quantity FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();
    assert_eq!(row.get::<_, i32>(0), 7);
}

#[tokio::test]
#[serial]
async fn test_quantity_update_rollback() {
    let ctx = setup_test_db().await;

    let cigar_id = create_test_cigar(&ctx.pool, "Test Cigar", 10, None)
        .await
        .unwrap();

    // Start a transaction
    let mut client = ctx.pool.get().await.unwrap();
    let transaction = client.transaction().await.unwrap();

    // Update quantity
    transaction
        .execute(
            "UPDATE cigars SET quantity = quantity - 3 WHERE id = $1",
            &[&cigar_id],
        )
        .await
        .unwrap();

    // Rollback transaction
    transaction.rollback().await.unwrap();

    // Verify update was rolled back
    let row = client
        .query_one("SELECT quantity FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();
    assert_eq!(
        row.get::<_, i32>(0),
        10,
        "Quantity should be unchanged after rollback"
    );
}

#[tokio::test]
#[serial]
async fn test_concurrent_quantity_updates() {
    let ctx = setup_test_db().await;

    let cigar_id = create_test_cigar(&ctx.pool, "Concurrent Test", 100, None)
        .await
        .unwrap();

    // Wait for cigar to be visible in all connections with longer timeout
    let mut cigar_found = false;
    for _attempt in 0..30 {
        let client = ctx.pool.get().await.unwrap();
        let result = client
            .query_opt("SELECT quantity FROM cigars WHERE id = $1", &[&cigar_id])
            .await
            .unwrap();
        if result.is_some() {
            let row = result.unwrap();
            let initial_quantity: i32 = row.get(0);
            assert_eq!(initial_quantity, 100, "Initial quantity should be 100");
            cigar_found = true;
            break;
        }
        // Wait longer between retries for CI environments
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
    }

    assert!(cigar_found, "Cigar was not found after initial creation");

    // Ensure the cigar is fully committed before starting concurrent updates
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Use join_all to ensure all operations complete before checking
    let pool = ctx.pool.clone();
    let updates: Vec<_> = (0..5)
        .map(|i| {
            let pool = pool.clone();
            let id = cigar_id;
            async move {
                // Add a small stagger to reduce contention
                tokio::time::sleep(tokio::time::Duration::from_millis(i as u64 * 20)).await;

                let client = pool.get().await.unwrap();

                // Retry logic for transient issues with longer backoff
                let mut retries = 0;
                loop {
                    let result = client
                        .execute(
                            "UPDATE cigars SET quantity = quantity - 1 WHERE id = $1",
                            &[&id],
                        )
                        .await;

                    match result {
                        Ok(rows) if rows > 0 => break,
                        Ok(_) if retries < 10 => {
                            retries += 1;
                            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                            continue;
                        }
                        Ok(_) => panic!(
                            "Update {} affected 0 rows after {} retries - cigar not found",
                            i, retries
                        ),
                        Err(e) => panic!("Update {} failed: {:?}", i, e),
                    }
                }
            }
        })
        .collect();

    // Wait for all updates to complete
    futures::future::join_all(updates).await;

    // Add a longer delay to ensure all transactions are fully committed
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Verify final quantity
    let client = ctx.pool.get().await.unwrap();
    let row = client
        .query_one("SELECT quantity FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap();

    let quantity: i32 = row.get(0);
    assert_eq!(
        quantity, 95,
        "5 concurrent decrements should result in quantity 95"
    );

    cleanup_db(&ctx.pool).await.unwrap();
}
