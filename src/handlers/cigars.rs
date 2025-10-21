use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::convert::Infallible;
use uuid::Uuid;
use warp::{Reply, Rejection};

use crate::{DbPool, models::*};

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