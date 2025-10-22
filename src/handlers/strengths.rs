use chrono::Utc;
use serde_json::json;
use uuid::Uuid;
use warp::{Reply, Rejection};

use crate::{DbPool, models::*, validation::Validate};

pub async fn get_strengths(db: DbPool) -> Result<impl Reply, Rejection> {
    match db.query(
        "SELECT id, name, description, level, created_at, updated_at FROM strengths ORDER BY level ASC",
        &[]
    ).await {
        Ok(rows) => {
            let mut strengths = Vec::new();
            for row in rows {
                let strength = Strength {
                    id: row.get(0),
                    name: row.get(1),
                    description: row.get(2),
                    level: row.get(3),
                    created_at: row.get(4),
                    updated_at: row.get(5),
                };
                strengths.push(strength);
            }
            Ok(warp::reply::json(&strengths))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to fetch strengths"})))
        }
    }
}

pub async fn create_strength(create_strength: CreateStrength, db: DbPool) -> Result<impl Reply, Rejection> {
    // Validate input
    create_strength.validate().map_err(warp::reject::custom)?;
    
    let id = Uuid::new_v4();
    let now = Utc::now();
    
    match db.query_one(
        "INSERT INTO strengths (id, name, description, level, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6) 
         RETURNING id, name, description, level, created_at, updated_at",
        &[&id, &create_strength.name, &create_strength.description, &create_strength.level, &now, &now]
    ).await {
        Ok(row) => {
            let strength = Strength {
                id: row.get(0),
                name: row.get(1),
                description: row.get(2),
                level: row.get(3),
                created_at: row.get(4),
                updated_at: row.get(5),
            };
            Ok(warp::reply::json(&strength))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to create strength"})))
        }
    }
}

pub async fn update_strength(id: Uuid, update_strength: UpdateStrength, db: DbPool) -> Result<impl Reply, Rejection> {
    // Validate input
    update_strength.validate().map_err(warp::reject::custom)?;
    
    let now = Utc::now();
    
    match db.query_one(
        "UPDATE strengths SET 
         name = COALESCE($2, name),
         description = COALESCE($3, description),
         level = COALESCE($4, level),
         updated_at = $5
         WHERE id = $1
         RETURNING id, name, description, level, created_at, updated_at",
        &[&id, &update_strength.name, &update_strength.description, &update_strength.level, &now]
    ).await {
        Ok(row) => {
            let strength = Strength {
                id: row.get(0),
                name: row.get(1),
                description: row.get(2),
                level: row.get(3),
                created_at: row.get(4),
                updated_at: row.get(5),
            };
            Ok(warp::reply::json(&strength))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to update strength"})))
        }
    }
}

pub async fn delete_strength(id: Uuid, db: DbPool) -> Result<impl Reply, Rejection> {
    match db.execute("DELETE FROM strengths WHERE id = $1", &[&id]).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok(warp::reply::json(&json!({"message": "Strength deleted successfully"})))
            } else {
                Ok(warp::reply::json(&json!({"error": "Strength not found"})))
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to delete strength"})))
        }
    }
}