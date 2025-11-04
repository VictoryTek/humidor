use tokio;
use tokio_postgres::{NoTls, Error};
use uuid::Uuid;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Connect to the database
    let (client, connection) = tokio_postgres::connect(
        "host=localhost port=5432 user=humidor password=humidor123 dbname=humidor",
        NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let cigar_id = Uuid::new_v4();
    let now = Utc::now();
    
    // Test 1: Try with Option<String> directly
    println!("Test 1: Trying with Option<String> directly...");
    let notes: Option<String> = Some("Test notes".to_string());
    match client
        .query_opt(
            "INSERT INTO wish_list (id, user_id, cigar_id, notes, created_at)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id",
            &[&id, &user_id, &cigar_id, &notes, &now],
        )
        .await
    {
        Ok(_) => println!("✓ Test 1 SUCCESS"),
        Err(e) => println!("✗ Test 1 FAILED: {}", e),
    }
    
    // Test 2: Try with Option<&str>
    println!("\nTest 2: Trying with Option<&str> using as_deref()...");
    let id2 = Uuid::new_v4();
    let notes2: Option<String> = Some("Test notes 2".to_string());
    let notes_ref: Option<&str> = notes2.as_deref();
    match client
        .query_opt(
            "INSERT INTO wish_list (id, user_id, cigar_id, notes, created_at)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id",
            &[&id2, &user_id, &cigar_id, &notes_ref, &now],
        )
        .await
    {
        Ok(_) => println!("✓ Test 2 SUCCESS"),
        Err(e) => println!("✗ Test 2 FAILED: {}", e),
    }
    
    // Test 3: Try with explicit None
    println!("\nTest 3: Trying with None...");
    let id3 = Uuid::new_v4();
    let cigar_id3 = Uuid::new_v4();
    let notes3: Option<&str> = None;
    match client
        .query_opt(
            "INSERT INTO wish_list (id, user_id, cigar_id, notes, created_at)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id",
            &[&id3, &user_id, &cigar_id3, &notes3, &now],
        )
        .await
    {
        Ok(_) => println!("✓ Test 3 SUCCESS"),
        Err(e) => println!("✗ Test 3 FAILED: {}", e),
    }
    
    // Test 4: Try with &Option<String>
    println!("\nTest 4: Trying with &Option<String>...");
    let id4 = Uuid::new_v4();
    let cigar_id4 = Uuid::new_v4();
    let notes4: Option<String> = Some("Test notes 4".to_string());
    match client
        .query_opt(
            "INSERT INTO wish_list (id, user_id, cigar_id, notes, created_at)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id",
            &[&id4, &user_id, &cigar_id4, &notes4 as &(dyn tokio_postgres::types::ToSql + Sync), &now],
        )
        .await
    {
        Ok(_) => println!("✓ Test 4 SUCCESS"),
        Err(e) => println!("✗ Test 4 FAILED: {}", e),
    }

    Ok(())
}
