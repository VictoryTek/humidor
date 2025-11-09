use chrono::Utc;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;
use warp::{Rejection, Reply};

use crate::{
    errors::AppError, middleware::auth::AuthContext, models::*, validation::Validate, DbPool,
};

#[derive(Debug, Serialize)]
pub struct CigarResponse {
    pub cigars: Vec<Cigar>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

pub async fn get_cigars(
    params: std::collections::HashMap<String, String>,
    _auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    use crate::errors::AppError;
    use tokio_postgres::types::ToSql;

    let start_time = std::time::Instant::now();

    // Acquire a connection from the pool
    let db = pool.get().await.map_err(|e| {
        warp::reject::custom(AppError::DatabaseError(format!(
            "Connection pool error: {}",
            e
        )))
    })?;

    // Parse pagination parameters
    let page = params
        .get("page")
        .and_then(|p| p.parse::<i64>().ok())
        .unwrap_or(1)
        .max(1); // Ensure page is at least 1

    let page_size = params
        .get("page_size")
        .and_then(|ps| ps.parse::<i64>().ok())
        .unwrap_or(50)
        .clamp(1, 100); // Allow 1-100 items per page

    let offset = (page - 1) * page_size;

    // Build query with parameterized conditions to prevent SQL injection
    let base_query = "SELECT id, humidor_id, brand_id, name, size_id, strength_id, origin_id, wrapper, binder, filler, price, purchase_date, notes, quantity, ring_gauge_id, length, image_url, retail_link, is_active, created_at, updated_at FROM cigars";
    let count_query = "SELECT COUNT(*) FROM cigars";
    let mut conditions = Vec::new();
    let mut param_values: Vec<Box<dyn ToSql + Sync + Send>> = Vec::new();
    let mut param_counter = 1;

    // Check for humidor_id filter
    if let Some(humidor_id_str) = params.get("humidor_id") {
        if let Ok(humidor_uuid) = Uuid::parse_str(humidor_id_str) {
            conditions.push(format!("humidor_id = ${}", param_counter));
            param_values.push(Box::new(humidor_uuid));
            param_counter += 1;
        }
    }

    // Check for organizer filters (brand, size, origin, strength, ring_gauge)
    if let Some(brand_id_str) = params.get("brand_id") {
        if let Ok(brand_uuid) = Uuid::parse_str(brand_id_str) {
            conditions.push(format!("brand_id = ${}", param_counter));
            param_values.push(Box::new(brand_uuid));
            param_counter += 1;
        }
    }
    if let Some(size_id_str) = params.get("size_id") {
        if let Ok(size_uuid) = Uuid::parse_str(size_id_str) {
            conditions.push(format!("size_id = ${}", param_counter));
            param_values.push(Box::new(size_uuid));
            param_counter += 1;
        }
    }
    if let Some(origin_id_str) = params.get("origin_id") {
        if let Ok(origin_uuid) = Uuid::parse_str(origin_id_str) {
            conditions.push(format!("origin_id = ${}", param_counter));
            param_values.push(Box::new(origin_uuid));
            param_counter += 1;
        }
    }
    if let Some(strength_id_str) = params.get("strength_id") {
        if let Ok(strength_uuid) = Uuid::parse_str(strength_id_str) {
            conditions.push(format!("strength_id = ${}", param_counter));
            param_values.push(Box::new(strength_uuid));
            param_counter += 1;
        }
    }
    if let Some(ring_gauge_id_str) = params.get("ring_gauge_id") {
        if let Ok(ring_gauge_uuid) = Uuid::parse_str(ring_gauge_id_str) {
            conditions.push(format!("ring_gauge_id = ${}", param_counter));
            param_values.push(Box::new(ring_gauge_uuid));
            param_counter += 1;
        }
    }

    // Build WHERE clause
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    // Get total count with same filters
    let total_count_query = format!("{}{}", count_query, where_clause);

    // Convert boxed parameters to references for query execution
    let param_refs: Vec<&(dyn ToSql + Sync)> = param_values
        .iter()
        .map(|b| &**b as &(dyn ToSql + Sync))
        .collect();

    // Execute count query
    let total = match db.query_one(&total_count_query, &param_refs[..]).await {
        Ok(row) => row.get::<_, i64>(0),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get total count");
            return Err(warp::reject::custom(AppError::DatabaseError(
                "Failed to count cigars".to_string(),
            )));
        }
    };

    let total_pages = (total as f64 / page_size as f64).ceil() as i64;

    // Add pagination parameters to the param list
    param_values.push(Box::new(page_size));
    param_values.push(Box::new(offset));

    // Rebuild param_refs with new values
    let param_refs: Vec<&(dyn ToSql + Sync)> = param_values
        .iter()
        .map(|b| &**b as &(dyn ToSql + Sync))
        .collect();

    // Build the final query with pagination
    let query = format!(
        "{}{} ORDER BY is_active DESC, created_at DESC LIMIT ${} OFFSET ${}",
        base_query,
        where_clause,
        param_counter,
        param_counter + 1
    );

    match db.query(&query, &param_refs[..]).await {
        Ok(rows) => {
            let mut cigars = Vec::new();
            for row in rows {
                let cigar = Cigar {
                    id: row.get(0),
                    humidor_id: row.get(1),
                    brand_id: row.get(2),
                    name: row.get(3),
                    size_id: row.get(4),
                    strength_id: row.get(5),
                    origin_id: row.get(6),
                    wrapper: row.get(7),
                    binder: row.get(8),
                    filler: row.get(9),
                    price: row.get(10),
                    purchase_date: row.get(11),
                    notes: row.get(12),
                    quantity: row.get(13),
                    ring_gauge_id: row.get(14),
                    length: row.get(15),
                    image_url: row.get(16),
                    retail_link: row.get(17),
                    is_active: row.get(18),
                    created_at: row.get(19),
                    updated_at: row.get(20),
                };
                cigars.push(cigar);
            }

            let elapsed = start_time.elapsed();
            if elapsed.as_millis() > 100 {
                tracing::warn!(
                    duration_ms = elapsed.as_millis(),
                    total_results = total,
                    page = page,
                    "Slow query detected in get_cigars"
                );
            } else {
                tracing::debug!(
                    duration_ms = elapsed.as_millis(),
                    total_results = total,
                    page = page,
                    "Query completed"
                );
            }

            let response = CigarResponse {
                cigars,
                total,
                page,
                page_size,
                total_pages,
            };

            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error fetching cigars");
            Ok(warp::reply::json(
                &json!({"error": "Failed to fetch cigars"}),
            ))
        }
    }
}

pub async fn create_cigar(
    create_cigar: CreateCigar,
    _auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    // Validate input
    create_cigar.validate().map_err(warp::reject::custom)?;

    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    match db.query_one(
        "INSERT INTO cigars (id, humidor_id, brand_id, name, size_id, strength_id, origin_id, wrapper, binder, filler, price, purchase_date, notes, quantity, ring_gauge_id, length, image_url, retail_link, is_active, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, true, $19, $20)
         RETURNING id, humidor_id, brand_id, name, size_id, strength_id, origin_id, wrapper, binder, filler, price, purchase_date, notes, quantity, ring_gauge_id, length, image_url, retail_link, is_active, created_at, updated_at",
        &[&id, &create_cigar.humidor_id, &create_cigar.brand_id, &create_cigar.name, &create_cigar.size_id, &create_cigar.strength_id, &create_cigar.origin_id,
          &create_cigar.wrapper, &create_cigar.binder, &create_cigar.filler, &create_cigar.price, &create_cigar.purchase_date,
          &create_cigar.notes, &create_cigar.quantity, &create_cigar.ring_gauge_id, &create_cigar.length, &create_cigar.image_url, &create_cigar.retail_link, &now, &now]
    ).await {
        Ok(row) => {
            let cigar = Cigar {
                id: row.get(0),
                humidor_id: row.get(1),
                brand_id: row.get(2),
                name: row.get(3),
                size_id: row.get(4),
                strength_id: row.get(5),
                origin_id: row.get(6),
                wrapper: row.get(7),
                binder: row.get(8),
                filler: row.get(9),
                price: row.get(10),
                purchase_date: row.get(11),
                notes: row.get(12),
                quantity: row.get(13),
                ring_gauge_id: row.get(14),
                length: row.get(15),
                image_url: row.get(16),
                retail_link: row.get(17),
                is_active: row.get(18),
                created_at: row.get(19),
                updated_at: row.get(20),
            };
            Ok(warp::reply::json(&cigar))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(&json!({"error": "Failed to create cigar"})))
        }
    }
}

pub async fn get_cigar(
    id: Uuid,
    _auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    match db.query_one(
        "SELECT id, humidor_id, brand_id, name, size_id, strength_id, origin_id, wrapper, binder, filler, price, purchase_date, notes, quantity, ring_gauge_id, length, image_url, retail_link, is_active, created_at, updated_at FROM cigars WHERE id = $1",
        &[&id]
    ).await {
        Ok(row) => {
            let cigar = Cigar {
                id: row.get(0),
                humidor_id: row.get(1),
                brand_id: row.get(2),
                name: row.get(3),
                size_id: row.get(4),
                strength_id: row.get(5),
                origin_id: row.get(6),
                wrapper: row.get(7),
                binder: row.get(8),
                filler: row.get(9),
                price: row.get(10),
                purchase_date: row.get(11),
                notes: row.get(12),
                quantity: row.get(13),
                ring_gauge_id: row.get(14),
                length: row.get(15),
                image_url: row.get(16),
                retail_link: row.get(17),
                is_active: row.get(18),
                created_at: row.get(19),
                updated_at: row.get(20),
            };
            Ok(warp::reply::json(&cigar))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(&json!({"error": "Cigar not found"})))
        }
    }
}

pub async fn update_cigar(
    id: Uuid,
    update_cigar: UpdateCigar,
    _auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    // Validate input
    update_cigar.validate().map_err(warp::reject::custom)?;

    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    let now = Utc::now();

    match db.query_one(
        "UPDATE cigars SET
         humidor_id = COALESCE($2, humidor_id),
         brand_id = COALESCE($3, brand_id),
         name = COALESCE($4, name),
         size_id = COALESCE($5, size_id),
         strength_id = COALESCE($6, strength_id),
         origin_id = COALESCE($7, origin_id),
         wrapper = COALESCE($8, wrapper),
         binder = COALESCE($9, binder),
         filler = COALESCE($10, filler),
         price = COALESCE($11, price),
         purchase_date = COALESCE($12, purchase_date),
         notes = COALESCE($13, notes),
         quantity = COALESCE($14, quantity),
         ring_gauge_id = COALESCE($15, ring_gauge_id),
         length = COALESCE($16, length),
         image_url = COALESCE($17, image_url),
         retail_link = COALESCE($18, retail_link),
         is_active = CASE
             WHEN $14 IS NOT NULL AND $14 = 0 THEN false
             WHEN $14 IS NOT NULL AND $14 > 0 THEN true
             ELSE is_active
         END,
         updated_at = $19
         WHERE id = $1
         RETURNING id, humidor_id, brand_id, name, size_id, strength_id, origin_id, wrapper, binder, filler, price, purchase_date, notes, quantity, ring_gauge_id, length, image_url, retail_link, is_active, created_at, updated_at",
        &[&id, &update_cigar.humidor_id, &update_cigar.brand_id, &update_cigar.name, &update_cigar.size_id, &update_cigar.strength_id, &update_cigar.origin_id,
          &update_cigar.wrapper, &update_cigar.binder, &update_cigar.filler, &update_cigar.price, &update_cigar.purchase_date,
          &update_cigar.notes, &update_cigar.quantity, &update_cigar.ring_gauge_id, &update_cigar.length, &update_cigar.image_url, &update_cigar.retail_link, &now]
    ).await {
        Ok(row) => {
            let cigar = Cigar {
                id: row.get(0),
                humidor_id: row.get(1),
                brand_id: row.get(2),
                name: row.get(3),
                size_id: row.get(4),
                strength_id: row.get(5),
                origin_id: row.get(6),
                wrapper: row.get(7),
                binder: row.get(8),
                filler: row.get(9),
                price: row.get(10),
                purchase_date: row.get(11),
                notes: row.get(12),
                quantity: row.get(13),
                ring_gauge_id: row.get(14),
                length: row.get(15),
                image_url: row.get(16),
                retail_link: row.get(17),
                is_active: row.get(18),
                created_at: row.get(19),
                updated_at: row.get(20),
            };
            Ok(warp::reply::json(&cigar))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(&json!({"error": "Failed to update cigar"})))
        }
    }
}

pub async fn delete_cigar(
    id: Uuid,
    _auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    // Hard delete: actually remove the cigar from the database
    // Note: favorites will keep snapshot data due to ON DELETE SET NULL on cigar_id
    match db.execute("DELETE FROM cigars WHERE id = $1", &[&id]).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok(warp::reply::json(
                    &json!({"message": "Cigar deleted successfully"}),
                ))
            } else {
                Ok(warp::reply::json(&json!({"error": "Cigar not found"})))
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(
                &json!({"error": "Failed to delete cigar"}),
            ))
        }
    }
}

#[derive(serde::Deserialize)]
pub struct ScrapeRequest {
    url: String,
}

pub async fn scrape_cigar_url(
    body: ScrapeRequest,
    _auth: AuthContext,
) -> Result<impl Reply, Rejection> {
    use crate::services::scrape_cigar_url;

    match scrape_cigar_url(&body.url).await {
        Ok(data) => Ok(warp::reply::json(&data)),
        Err(e) => {
            tracing::error!(error = %e, "Scraping error");
            Ok(warp::reply::json(
                &json!({"error": "Failed to scrape cigar information"}),
            ))
        }
    }
}
