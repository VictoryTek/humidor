use chrono::Utc;
use serde_json::json;
use uuid::Uuid;
use warp::{Rejection, Reply};

use crate::{DbPool, errors::AppError, middleware::AuthContext, models::*, validation::Validate};

pub async fn get_strengths(auth: AuthContext, pool: DbPool) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    match db.query(
        "SELECT id, user_id, name, level, description, created_at, updated_at FROM strengths WHERE user_id = $1 ORDER BY level ASC",
        &[&auth.user_id]
    ).await {
        Ok(rows) => {
            let mut strengths = Vec::new();
            for row in rows {
                let strength = Strength {
                    id: row.get(0),
                    user_id: row.get(1),
                    name: row.get(2),
                    level: row.get(3),
                    description: row.get(4),
                    created_at: row.get(5),
                    updated_at: row.get(6),
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
    auth: AuthContext,
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
            "INSERT INTO strengths (id, user_id, name, level, description, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7) 
         RETURNING id, user_id, name, level, description, created_at, updated_at",
            &[
                &id,
                &auth.user_id,
                &create_strength.name,
                &create_strength.level,
                &create_strength.description,
                &now,
                &now,
            ],
        )
        .await
    {
        Ok(row) => {
            let strength = Strength {
                id: row.get(0),
                user_id: row.get(1),
                name: row.get(2),
                level: row.get(3),
                description: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
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
    auth: AuthContext,
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
        .query_opt(
            "UPDATE strengths SET 
         name = COALESCE($2, name),
         level = COALESCE($3, level),
         description = COALESCE($4, description),
         updated_at = $5
         WHERE id = $1 AND user_id = $6
         RETURNING id, user_id, name, level, description, created_at, updated_at",
            &[
                &id,
                &update_strength.name,
                &update_strength.level,
                &update_strength.description,
                &now,
                &auth.user_id,
            ],
        )
        .await
    {
        Ok(Some(row)) => {
            let strength = Strength {
                id: row.get(0),
                user_id: row.get(1),
                name: row.get(2),
                level: row.get(3),
                description: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
            };
            Ok(warp::reply::json(&strength))
        }
        Ok(None) => Ok(warp::reply::json(
            &json!({"error": "Strength not found or unauthorized"}),
        )),
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(
                &json!({"error": "Failed to update strength"}),
            ))
        }
    }
}

pub async fn delete_strength(
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
            "DELETE FROM strengths WHERE id = $1 AND user_id = $2",
            &[&id, &auth.user_id],
        )
        .await
    {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok(warp::reply::json(
                    &json!({"message": "Strength deleted successfully"}),
                ))
            } else {
                Ok(warp::reply::json(
                    &json!({"error": "Strength not found or unauthorized"}),
                ))
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
