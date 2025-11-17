use chrono::Utc;
use serde_json::json;
use uuid::Uuid;
use warp::{Rejection, Reply};

use crate::{DbPool, errors::AppError, middleware::AuthContext, models::*, validation::Validate};

pub async fn get_brands(auth: AuthContext, pool: DbPool) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    match db.query(
        "SELECT id, user_id, name, description, country, website, created_at, updated_at FROM brands WHERE user_id = $1 ORDER BY name ASC",
        &[&auth.user_id]
    ).await {
        Ok(rows) => {
            let mut brands = Vec::new();
            for row in rows {
                let brand = Brand {
                    id: row.get(0),
                    user_id: row.get(1),
                    name: row.get(2),
                    description: row.get(3),
                    country: row.get(4),
                    website: row.get(5),
                    created_at: row.get(6),
                    updated_at: row.get(7),
                };
                brands.push(brand);
            }
            Ok(warp::reply::json(&brands))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error fetching brands");
            Ok(warp::reply::json(&json!({"error": "Failed to fetch brands"})))
        }
    }
}

pub async fn create_brand(
    auth: AuthContext,
    create_brand: CreateBrand,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    // Validate input
    create_brand.validate().map_err(warp::reject::custom)?;

    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    match db
        .query_one(
            "INSERT INTO brands (id, user_id, name, description, country, website, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
         RETURNING id, user_id, name, description, country, website, created_at, updated_at",
            &[
                &id,
                &auth.user_id,
                &create_brand.name,
                &create_brand.description,
                &create_brand.country,
                &create_brand.website,
                &now,
                &now,
            ],
        )
        .await
    {
        Ok(row) => {
            let brand = Brand {
                id: row.get(0),
                user_id: row.get(1),
                name: row.get(2),
                description: row.get(3),
                country: row.get(4),
                website: row.get(5),
                created_at: row.get(6),
                updated_at: row.get(7),
            };
            Ok(warp::reply::json(&brand))
        }
        Err(e) => {
            tracing::error!(error = %e, brand_name = %create_brand.name, "Database error creating brand");
            Ok(warp::reply::json(
                &json!({"error": "Failed to create brand"}),
            ))
        }
    }
}

pub async fn update_brand(
    id: Uuid,
    auth: AuthContext,
    update_brand: UpdateBrand,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    // Validate input
    update_brand.validate().map_err(warp::reject::custom)?;

    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    let now = Utc::now();

    match db
        .query_opt(
            "UPDATE brands SET 
         name = COALESCE($2, name),
         description = COALESCE($3, description),
         country = COALESCE($4, country),
         website = COALESCE($5, website),
         updated_at = $6
         WHERE id = $1 AND user_id = $7
         RETURNING id, user_id, name, description, country, website, created_at, updated_at",
            &[
                &id,
                &update_brand.name,
                &update_brand.description,
                &update_brand.country,
                &update_brand.website,
                &now,
                &auth.user_id,
            ],
        )
        .await
    {
        Ok(Some(row)) => {
            let brand = Brand {
                id: row.get(0),
                user_id: row.get(1),
                name: row.get(2),
                description: row.get(3),
                country: row.get(4),
                website: row.get(5),
                created_at: row.get(6),
                updated_at: row.get(7),
            };
            Ok(warp::reply::json(&brand))
        }
        Ok(None) => Ok(warp::reply::json(
            &json!({"error": "Brand not found or unauthorized"}),
        )),
        Err(e) => {
            tracing::error!(error = %e, brand_id = %id, "Database error updating brand");
            Ok(warp::reply::json(
                &json!({"error": "Failed to update brand"}),
            ))
        }
    }
}

pub async fn delete_brand(
    id: Uuid,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    match db
        .execute(
            "DELETE FROM brands WHERE id = $1 AND user_id = $2",
            &[&id, &auth.user_id],
        )
        .await
    {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok(warp::reply::json(
                    &json!({"message": "Brand deleted successfully"}),
                ))
            } else {
                Ok(warp::reply::json(
                    &json!({"error": "Brand not found or unauthorized"}),
                ))
            }
        }
        Err(e) => {
            tracing::error!(error = %e, brand_id = %id, "Database error deleting brand");
            Ok(warp::reply::json(
                &json!({"error": "Failed to delete brand"}),
            ))
        }
    }
}
