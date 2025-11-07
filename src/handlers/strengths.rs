use chrono::Utc;
use serde_json::json;
use uuid::Uuid;
use warp::{Rejection, Reply};

use crate::{errors::AppError, models::*, validation::Validate, DbPool};

pub async fn get_strengths(pool: DbPool) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

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
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(&json!({"error": "Failed to fetch strengths"})))
        }
    }
}

pub async fn create_strength(
    create_strength: CreateStrength,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    // Validate input
    create_strength.validate().map_err(warp::reject::custom)?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    match db
        .query_one(
            "INSERT INTO strengths (id, name, description, level, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6) 
         RETURNING id, name, description, level, created_at, updated_at",
            &[
                &id,
                &create_strength.name,
                &create_strength.description,
                &create_strength.level,
                &now,
                &now,
            ],
        )
        .await
    {
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
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(
                &json!({"error": "Failed to create strength"}),
            ))
        }
    }
}

pub async fn update_strength(
    id: Uuid,
    update_strength: UpdateStrength,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    // Validate input
    update_strength.validate().map_err(warp::reject::custom)?;

    let now = Utc::now();

    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    match db
        .query_one(
            "UPDATE strengths SET 
         name = COALESCE($2, name),
         description = COALESCE($3, description),
         level = COALESCE($4, level),
         updated_at = $5
         WHERE id = $1
         RETURNING id, name, description, level, created_at, updated_at",
            &[
                &id,
                &update_strength.name,
                &update_strength.description,
                &update_strength.level,
                &now,
            ],
        )
        .await
    {
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
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(
                &json!({"error": "Failed to update strength"}),
            ))
        }
    }
}

pub async fn delete_strength(id: Uuid, pool: DbPool) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    match db
        .execute("DELETE FROM strengths WHERE id = $1", &[&id])
        .await
    {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok(warp::reply::json(
                    &json!({"message": "Strength deleted successfully"}),
                ))
            } else {
                Ok(warp::reply::json(&json!({"error": "Strength not found"})))
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(
                &json!({"error": "Failed to delete strength"}),
            ))
        }
    }
}
