use chrono::Utc;
use serde_json::json;
use uuid::Uuid;
use warp::{Rejection, Reply};

use crate::{DbPool, errors::AppError, middleware::AuthContext, models::*, validation::Validate};

pub async fn get_sizes(auth: AuthContext, pool: DbPool) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    match db.query(
        "SELECT id, user_id, name, length_inches, ring_gauge, description, created_at, updated_at FROM sizes WHERE user_id = $1 ORDER BY name ASC",
        &[&auth.user_id]
    ).await {
        Ok(rows) => {
            let mut sizes = Vec::new();
            for row in rows {
                let size = Size {
                    id: row.get(0),
                    user_id: row.get(1),
                    name: row.get(2),
                    length_inches: row.get(3),
                    ring_gauge: row.get(4),
                    description: row.get(5),
                    created_at: row.get(6),
                    updated_at: row.get(7),
                };
                sizes.push(size);
            }
            Ok(warp::reply::json(&sizes))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(&json!({"error": "Failed to fetch sizes"})))
        }
    }
}

pub async fn create_size(
    auth: AuthContext,
    create_size: CreateSize,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    // Validate input
    create_size.validate().map_err(warp::reject::custom)?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    match db.query_one(
        "INSERT INTO sizes (id, user_id, name, length_inches, ring_gauge, description, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
         RETURNING id, user_id, name, length_inches, ring_gauge, description, created_at, updated_at",
        &[&id, &auth.user_id, &create_size.name, &create_size.length_inches, &create_size.ring_gauge, &create_size.description, &now, &now]
    ).await {
        Ok(row) => {
            let size = Size {
                id: row.get(0),
                user_id: row.get(1),
                name: row.get(2),
                length_inches: row.get(3),
                ring_gauge: row.get(4),
                description: row.get(5),
                created_at: row.get(6),
                updated_at: row.get(7),
            };
            Ok(warp::reply::json(&size))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(&json!({"error": "Failed to create size"})))
        }
    }
}

pub async fn update_size(
    id: Uuid,
    auth: AuthContext,
    update_size: UpdateSize,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    // Validate input
    update_size.validate().map_err(warp::reject::custom)?;

    let now = Utc::now();

    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    match db
        .query_opt(
            "UPDATE sizes SET 
         name = COALESCE($2, name),
         length_inches = COALESCE($3, length_inches),
         ring_gauge = COALESCE($4, ring_gauge),
         description = COALESCE($5, description),
         updated_at = $6
         WHERE id = $1 AND user_id = $7
         RETURNING id, user_id, name, length_inches, ring_gauge, description, created_at, updated_at",
            &[
                &id,
                &update_size.name,
                &update_size.length_inches,
                &update_size.ring_gauge,
                &update_size.description,
                &now,
                &auth.user_id,
            ],
        )
        .await
    {
        Ok(Some(row)) => {
            let size = Size {
                id: row.get(0),
                user_id: row.get(1),
                name: row.get(2),
                length_inches: row.get(3),
                ring_gauge: row.get(4),
                description: row.get(5),
                created_at: row.get(6),
                updated_at: row.get(7),
            };
            Ok(warp::reply::json(&size))
        }
        Ok(None) => {
            Ok(warp::reply::json(&json!({"error": "Size not found or unauthorized"})))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(
                &json!({"error": "Failed to update size"}),
            ))
        }
    }
}

pub async fn delete_size(
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
            "DELETE FROM sizes WHERE id = $1 AND user_id = $2",
            &[&id, &auth.user_id],
        )
        .await
    {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok(warp::reply::json(
                    &json!({"message": "Size deleted successfully"}),
                ))
            } else {
                Ok(warp::reply::json(
                    &json!({"error": "Size not found or unauthorized"}),
                ))
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(
                &json!({"error": "Failed to delete size"}),
            ))
        }
    }
}
