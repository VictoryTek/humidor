use chrono::Utc;
use serde_json::json;
use std::convert::Infallible;
use uuid::Uuid;
use warp::{Reply, Rejection};

use crate::{DbPool, models::*};

pub async fn get_sizes(db: DbPool) -> Result<impl Reply, Rejection> {
    match db.query(
        "SELECT id, name, length_inches, ring_gauge, description, created_at, updated_at FROM sizes ORDER BY name ASC",
        &[]
    ).await {
        Ok(rows) => {
            let mut sizes = Vec::new();
            for row in rows {
                let size = Size {
                    id: row.get(0),
                    name: row.get(1),
                    length_inches: row.get(2),
                    ring_gauge: row.get(3),
                    description: row.get(4),
                    created_at: row.get(5),
                    updated_at: row.get(6),
                };
                sizes.push(size);
            }
            Ok(warp::reply::json(&sizes))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to fetch sizes"})))
        }
    }
}

pub async fn create_size(create_size: CreateSize, db: DbPool) -> Result<impl Reply, Rejection> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    
    match db.query_one(
        "INSERT INTO sizes (id, name, length_inches, ring_gauge, description, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7) 
         RETURNING id, name, length_inches, ring_gauge, description, created_at, updated_at",
        &[&id, &create_size.name, &create_size.length_inches, &create_size.ring_gauge, &create_size.description, &now, &now]
    ).await {
        Ok(row) => {
            let size = Size {
                id: row.get(0),
                name: row.get(1),
                length_inches: row.get(2),
                ring_gauge: row.get(3),
                description: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
            };
            Ok(warp::reply::json(&size))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to create size"})))
        }
    }
}

pub async fn update_size(id: Uuid, update_size: UpdateSize, db: DbPool) -> Result<impl Reply, Rejection> {
    let now = Utc::now();
    
    match db.query_one(
        "UPDATE sizes SET 
         name = COALESCE($2, name),
         length_inches = COALESCE($3, length_inches),
         ring_gauge = COALESCE($4, ring_gauge),
         description = COALESCE($5, description),
         updated_at = $6
         WHERE id = $1
         RETURNING id, name, length_inches, ring_gauge, description, created_at, updated_at",
        &[&id, &update_size.name, &update_size.length_inches, &update_size.ring_gauge, &update_size.description, &now]
    ).await {
        Ok(row) => {
            let size = Size {
                id: row.get(0),
                name: row.get(1),
                length_inches: row.get(2),
                ring_gauge: row.get(3),
                description: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
            };
            Ok(warp::reply::json(&size))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to update size"})))
        }
    }
}

pub async fn delete_size(id: Uuid, db: DbPool) -> Result<impl Reply, Rejection> {
    match db.execute("DELETE FROM sizes WHERE id = $1", &[&id]).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok(warp::reply::json(&json!({"message": "Size deleted successfully"})))
            } else {
                Ok(warp::reply::json(&json!({"error": "Size not found"})))
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to delete size"})))
        }
    }
}
