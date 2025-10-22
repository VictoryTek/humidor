use chrono::Utc;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;
use warp::{Reply, Rejection};

use crate::{DbPool, models::*, validation::Validate};

#[derive(Debug, Serialize)]
pub struct CigarResponse {
    pub cigars: Vec<Cigar>,
    pub total: i64,
}

pub async fn get_cigars(db: DbPool) -> Result<impl Reply, Rejection> {
    match db.query("SELECT id, brand, name, size, strength, origin, wrapper, binder, filler, price, purchase_date, notes, quantity, humidor_location, created_at, updated_at FROM cigars ORDER BY created_at DESC LIMIT 50", &[]).await {
        Ok(rows) => {
            let mut cigars = Vec::new();
            for row in rows {
                let cigar = Cigar {
                    id: row.get(0),
                    brand: row.get(1),
                    name: row.get(2),
                    size: row.get(3),
                    strength: row.get(4),
                    origin: row.get(5),
                    wrapper: row.get(6),
                    binder: row.get(7),
                    filler: row.get(8),
                    price: row.get(9),
                    purchase_date: row.get(10),
                    notes: row.get(11),
                    quantity: row.get(12),
                    humidor_location: row.get(13),
                    created_at: row.get(14),
                    updated_at: row.get(15),
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

pub async fn create_cigar(create_cigar: CreateCigar, db: DbPool) -> Result<impl Reply, Rejection> {
    // Validate input
    create_cigar.validate().map_err(warp::reject::custom)?;
    
    let id = Uuid::new_v4();
    let now = Utc::now();
    
    match db.query_one(
        "INSERT INTO cigars (id, brand, name, size, strength, origin, wrapper, binder, filler, price, purchase_date, notes, quantity, humidor_location, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16) 
         RETURNING id, brand, name, size, strength, origin, wrapper, binder, filler, price, purchase_date, notes, quantity, humidor_location, created_at, updated_at",
        &[&id, &create_cigar.brand, &create_cigar.name, &create_cigar.size, &create_cigar.strength, &create_cigar.origin, 
          &create_cigar.wrapper, &create_cigar.binder, &create_cigar.filler, &create_cigar.price, &create_cigar.purchase_date, 
          &create_cigar.notes, &create_cigar.quantity, &create_cigar.humidor_location, &now, &now]
    ).await {
        Ok(row) => {
            let cigar = Cigar {
                id: row.get(0),
                brand: row.get(1),
                name: row.get(2),
                size: row.get(3),
                strength: row.get(4),
                origin: row.get(5),
                wrapper: row.get(6),
                binder: row.get(7),
                filler: row.get(8),
                price: row.get(9),
                purchase_date: row.get(10),
                notes: row.get(11),
                quantity: row.get(12),
                humidor_location: row.get(13),
                created_at: row.get(14),
                updated_at: row.get(15),
            };
            Ok(warp::reply::json(&cigar))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to create cigar"})))
        }
    }
}

pub async fn get_cigar(id: Uuid, db: DbPool) -> Result<impl Reply, Rejection> {
    match db.query_one(
        "SELECT id, brand, name, size, strength, origin, wrapper, binder, filler, price, purchase_date, notes, quantity, humidor_location, created_at, updated_at FROM cigars WHERE id = $1",
        &[&id]
    ).await {
        Ok(row) => {
            let cigar = Cigar {
                id: row.get(0),
                brand: row.get(1),
                name: row.get(2),
                size: row.get(3),
                strength: row.get(4),
                origin: row.get(5),
                wrapper: row.get(6),
                binder: row.get(7),
                filler: row.get(8),
                price: row.get(9),
                purchase_date: row.get(10),
                notes: row.get(11),
                quantity: row.get(12),
                humidor_location: row.get(13),
                created_at: row.get(14),
                updated_at: row.get(15),
            };
            Ok(warp::reply::json(&cigar))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Cigar not found"})))
        }
    }
}

pub async fn update_cigar(id: Uuid, update_cigar: UpdateCigar, db: DbPool) -> Result<impl Reply, Rejection> {
    // Validate input
    update_cigar.validate().map_err(warp::reject::custom)?;
    
    let now = Utc::now();
    
    match db.query_one(
        "UPDATE cigars SET 
         brand = COALESCE($2, brand),
         name = COALESCE($3, name),
         size = COALESCE($4, size),
         strength = COALESCE($5, strength),
         origin = COALESCE($6, origin),
         wrapper = COALESCE($7, wrapper),
         binder = COALESCE($8, binder),
         filler = COALESCE($9, filler),
         price = COALESCE($10, price),
         purchase_date = COALESCE($11, purchase_date),
         notes = COALESCE($12, notes),
         quantity = COALESCE($13, quantity),
         humidor_location = COALESCE($14, humidor_location),
         updated_at = $15
         WHERE id = $1
         RETURNING id, brand, name, size, strength, origin, wrapper, binder, filler, price, purchase_date, notes, quantity, humidor_location, created_at, updated_at",
        &[&id, &update_cigar.brand, &update_cigar.name, &update_cigar.size, &update_cigar.strength, &update_cigar.origin,
          &update_cigar.wrapper, &update_cigar.binder, &update_cigar.filler, &update_cigar.price, &update_cigar.purchase_date,
          &update_cigar.notes, &update_cigar.quantity, &update_cigar.humidor_location, &now]
    ).await {
        Ok(row) => {
            let cigar = Cigar {
                id: row.get(0),
                brand: row.get(1),
                name: row.get(2),
                size: row.get(3),
                strength: row.get(4),
                origin: row.get(5),
                wrapper: row.get(6),
                binder: row.get(7),
                filler: row.get(8),
                price: row.get(9),
                purchase_date: row.get(10),
                notes: row.get(11),
                quantity: row.get(12),
                humidor_location: row.get(13),
                created_at: row.get(14),
                updated_at: row.get(15),
            };
            Ok(warp::reply::json(&cigar))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to update cigar"})))
        }
    }
}

pub async fn delete_cigar(id: Uuid, db: DbPool) -> Result<impl Reply, Rejection> {
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