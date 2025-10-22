use chrono::Utc;
use serde_json::json;
use uuid::Uuid;
use warp::{Reply, Rejection};

use crate::{DbPool, models::*, validation::Validate};

pub async fn get_brands(db: DbPool) -> Result<impl Reply, Rejection> {
    match db.query(
        "SELECT id, name, description, country, website, created_at, updated_at FROM brands ORDER BY name ASC",
        &[]
    ).await {
        Ok(rows) => {
            let mut brands = Vec::new();
            for row in rows {
                let brand = Brand {
                    id: row.get(0),
                    name: row.get(1),
                    description: row.get(2),
                    country: row.get(3),
                    website: row.get(4),
                    created_at: row.get(5),
                    updated_at: row.get(6),
                };
                brands.push(brand);
            }
            Ok(warp::reply::json(&brands))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to fetch brands"})))
        }
    }
}

pub async fn create_brand(create_brand: CreateBrand, db: DbPool) -> Result<impl Reply, Rejection> {
    // Validate input
    create_brand.validate().map_err(warp::reject::custom)?;
    
    let id = Uuid::new_v4();
    let now = Utc::now();
    
    match db.query_one(
        "INSERT INTO brands (id, name, description, country, website, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7) 
         RETURNING id, name, description, country, website, created_at, updated_at",
        &[&id, &create_brand.name, &create_brand.description, &create_brand.country, &create_brand.website, &now, &now]
    ).await {
        Ok(row) => {
            let brand = Brand {
                id: row.get(0),
                name: row.get(1),
                description: row.get(2),
                country: row.get(3),
                website: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
            };
            Ok(warp::reply::json(&brand))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to create brand"})))
        }
    }
}

pub async fn update_brand(id: Uuid, update_brand: UpdateBrand, db: DbPool) -> Result<impl Reply, Rejection> {
    // Validate input
    update_brand.validate().map_err(warp::reject::custom)?;
    
    let now = Utc::now();
    
    match db.query_one(
        "UPDATE brands SET 
         name = COALESCE($2, name),
         description = COALESCE($3, description),
         country = COALESCE($4, country),
         website = COALESCE($5, website),
         updated_at = $6
         WHERE id = $1
         RETURNING id, name, description, country, website, created_at, updated_at",
        &[&id, &update_brand.name, &update_brand.description, &update_brand.country, &update_brand.website, &now]
    ).await {
        Ok(row) => {
            let brand = Brand {
                id: row.get(0),
                name: row.get(1),
                description: row.get(2),
                country: row.get(3),
                website: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
            };
            Ok(warp::reply::json(&brand))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to update brand"})))
        }
    }
}

pub async fn delete_brand(id: Uuid, db: DbPool) -> Result<impl Reply, Rejection> {
    match db.execute("DELETE FROM brands WHERE id = $1", &[&id]).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok(warp::reply::json(&json!({"message": "Brand deleted successfully"})))
            } else {
                Ok(warp::reply::json(&json!({"error": "Brand not found"})))
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to delete brand"})))
        }
    }
}
