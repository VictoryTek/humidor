use chrono::Utc;
use serde_json::json;
use std::convert::Infallible;
use uuid::Uuid;
use warp::{Reply, Rejection};

use crate::{DbPool, models::*};

pub async fn get_origins(db: DbPool) -> Result<impl Reply, Rejection> {
    match db.query(
        "SELECT id, name, country, region, description, created_at, updated_at FROM origins ORDER BY country ASC",
        &[]
    ).await {
        Ok(rows) => {
            let mut origins = Vec::new();
            for row in rows {
                let origin = Origin {
                    id: row.get(0),
                    name: row.get(1),
                    country: row.get(2),
                    region: row.get(3),
                    description: row.get(4),
                    created_at: row.get(5),
                    updated_at: row.get(6),
                };
                origins.push(origin);
            }
            Ok(warp::reply::json(&origins))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to fetch origins"})))
        }
    }
}

pub async fn create_origin(create_origin: CreateOrigin, db: DbPool) -> Result<impl Reply, Rejection> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    
    match db.query_one(
        "INSERT INTO origins (id, name, country, region, description, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7) 
         RETURNING id, name, country, region, description, created_at, updated_at",
        &[&id, &create_origin.name, &create_origin.country, &create_origin.region, &create_origin.description, &now, &now]
    ).await {
        Ok(row) => {
            let origin = Origin {
                id: row.get(0),
                name: row.get(1),
                country: row.get(2),
                region: row.get(3),
                description: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
            };
            Ok(warp::reply::json(&origin))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to create origin"})))
        }
    }
}

pub async fn update_origin(id: Uuid, update_origin: UpdateOrigin, db: DbPool) -> Result<impl Reply, Rejection> {
    let now = Utc::now();
    
    match db.query_one(
        "UPDATE origins SET 
         name = COALESCE($2, name),
         country = COALESCE($3, country),
         region = COALESCE($4, region),
         description = COALESCE($5, description),
         updated_at = $6
         WHERE id = $1
         RETURNING id, name, country, region, description, created_at, updated_at",
        &[&id, &update_origin.name, &update_origin.country, &update_origin.region, &update_origin.description, &now]
    ).await {
        Ok(row) => {
            let origin = Origin {
                id: row.get(0),
                name: row.get(1),
                country: row.get(2),
                region: row.get(3),
                description: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
            };
            Ok(warp::reply::json(&origin))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to update origin"})))
        }
    }
}

pub async fn delete_origin(id: Uuid, db: DbPool) -> Result<impl Reply, Rejection> {
    match db.execute("DELETE FROM origins WHERE id = $1", &[&id]).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok(warp::reply::json(&json!({"message": "Origin deleted successfully"})))
            } else {
                Ok(warp::reply::json(&json!({"error": "Origin not found"})))
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to delete origin"})))
        }
    }
}