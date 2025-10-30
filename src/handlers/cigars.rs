use chrono::Utc;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;
use warp::{Reply, Rejection};

use crate::{DbPool, models::*, validation::Validate, middleware::auth::AuthContext};

#[derive(Debug, Serialize)]
pub struct CigarResponse {
    pub cigars: Vec<Cigar>,
    pub total: i64,
}

pub async fn get_cigars(
    params: std::collections::HashMap<String, String>,
    _auth: AuthContext,
    db: DbPool
) -> Result<impl Reply, Rejection> {
    // Build query based on parameters
    let mut query = String::from("SELECT id, humidor_id, brand_id, name, size_id, strength_id, origin_id, wrapper, binder, filler, price, purchase_date, notes, quantity, ring_gauge_id, length, image_url, created_at, updated_at FROM cigars");
    let mut conditions = Vec::new();
    
    // Check for humidor_id filter
    if let Some(humidor_id) = params.get("humidor_id") {
        conditions.push(format!("humidor_id::text = '{}'", humidor_id));
    }
    
    // Check for organizer filters (brand, size, origin, strength, ring_gauge)
    if let Some(brand_id) = params.get("brand_id") {
        conditions.push(format!("brand_id::text = '{}'", brand_id));
    }
    if let Some(size_id) = params.get("size_id") {
        conditions.push(format!("size_id::text = '{}'", size_id));
    }
    if let Some(origin_id) = params.get("origin_id") {
        conditions.push(format!("origin_id::text = '{}'", origin_id));
    }
    if let Some(strength_id) = params.get("strength_id") {
        conditions.push(format!("strength_id::text = '{}'", strength_id));
    }
    if let Some(ring_gauge_id) = params.get("ring_gauge_id") {
        conditions.push(format!("ring_gauge_id::text = '{}'", ring_gauge_id));
    }
    
    // Add WHERE clause if there are conditions
    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }
    
    query.push_str(" ORDER BY created_at DESC LIMIT 50");
    
    match db.query(&query, &[]).await {
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
                    created_at: row.get(17),
                    updated_at: row.get(18),
                };
                cigars.push(cigar);
            }
            
            let total = cigars.len() as i64;
            let response = CigarResponse {
                cigars,
                total,
            };
            
            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to fetch cigars"})))
        }
    }
}

pub async fn create_cigar(create_cigar: CreateCigar, _auth: AuthContext, db: DbPool) -> Result<impl Reply, Rejection> {
    // Validate input
    create_cigar.validate().map_err(warp::reject::custom)?;
    
    let id = Uuid::new_v4();
    let now = Utc::now();
    
    match db.query_one(
        "INSERT INTO cigars (id, humidor_id, brand_id, name, size_id, strength_id, origin_id, wrapper, binder, filler, price, purchase_date, notes, quantity, ring_gauge_id, length, image_url, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19) 
         RETURNING id, humidor_id, brand_id, name, size_id, strength_id, origin_id, wrapper, binder, filler, price, purchase_date, notes, quantity, ring_gauge_id, length, image_url, created_at, updated_at",
        &[&id, &create_cigar.humidor_id, &create_cigar.brand_id, &create_cigar.name, &create_cigar.size_id, &create_cigar.strength_id, &create_cigar.origin_id, 
          &create_cigar.wrapper, &create_cigar.binder, &create_cigar.filler, &create_cigar.price, &create_cigar.purchase_date, 
          &create_cigar.notes, &create_cigar.quantity, &create_cigar.ring_gauge_id, &create_cigar.length, &create_cigar.image_url, &now, &now]
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
                created_at: row.get(17),
                updated_at: row.get(18),
            };
            Ok(warp::reply::json(&cigar))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to create cigar"})))
        }
    }
}

pub async fn get_cigar(id: Uuid, _auth: AuthContext, db: DbPool) -> Result<impl Reply, Rejection> {
    match db.query_one(
        "SELECT id, humidor_id, brand_id, name, size_id, strength_id, origin_id, wrapper, binder, filler, price, purchase_date, notes, quantity, ring_gauge_id, length, image_url, created_at, updated_at FROM cigars WHERE id = $1",
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
                created_at: row.get(17),
                updated_at: row.get(18),
            };
            Ok(warp::reply::json(&cigar))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Cigar not found"})))
        }
    }
}

pub async fn update_cigar(id: Uuid, update_cigar: UpdateCigar, _auth: AuthContext, db: DbPool) -> Result<impl Reply, Rejection> {
    // Validate input
    update_cigar.validate().map_err(warp::reject::custom)?;
    
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
         updated_at = $18
         WHERE id = $1
         RETURNING id, humidor_id, brand_id, name, size_id, strength_id, origin_id, wrapper, binder, filler, price, purchase_date, notes, quantity, ring_gauge_id, length, image_url, created_at, updated_at",
        &[&id, &update_cigar.humidor_id, &update_cigar.brand_id, &update_cigar.name, &update_cigar.size_id, &update_cigar.strength_id, &update_cigar.origin_id,
          &update_cigar.wrapper, &update_cigar.binder, &update_cigar.filler, &update_cigar.price, &update_cigar.purchase_date,
          &update_cigar.notes, &update_cigar.quantity, &update_cigar.ring_gauge_id, &update_cigar.length, &update_cigar.image_url, &now]
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
                created_at: row.get(17),
                updated_at: row.get(18),
            };
            Ok(warp::reply::json(&cigar))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to update cigar"})))
        }
    }
}

pub async fn delete_cigar(id: Uuid, _auth: AuthContext, db: DbPool) -> Result<impl Reply, Rejection> {
    match db.execute("DELETE FROM cigars WHERE id = $1", &[&id]).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok(warp::reply::json(&json!({"message": "Cigar deleted successfully"})))
            } else {
                Ok(warp::reply::json(&json!({"error": "Cigar not found"})))
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to delete cigar"})))
        }
    }
}

#[derive(serde::Deserialize)]
pub struct ScrapeRequest {
    url: String,
}

pub async fn scrape_cigar_url(body: ScrapeRequest, _auth: AuthContext) -> Result<impl Reply, Rejection> {
    use crate::services::scrape_cigar_url;
    
    match scrape_cigar_url(&body.url).await {
        Ok(data) => Ok(warp::reply::json(&data)),
        Err(e) => {
            eprintln!("Scraping error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to scrape cigar information"})))
        }
    }
}