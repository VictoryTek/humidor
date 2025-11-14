use chrono::Utc;
use serde_json::json;
use uuid::Uuid;
use warp::{Rejection, Reply};

use crate::{errors::AppError, middleware::AuthContext, models::*, validation::Validate, DbPool};

pub async fn get_origins(auth: AuthContext, pool: DbPool) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    match db.query(
        "SELECT id, user_id, name, country, region, description, created_at, updated_at FROM origins WHERE user_id = $1 ORDER BY country ASC",
        &[&auth.user_id]
    ).await {
        Ok(rows) => {
            let mut origins = Vec::new();
            for row in rows {
                let origin = Origin {
                    id: row.get(0),
                    user_id: row.get(1),
                    name: row.get(2),
                    country: row.get(3),
                    region: row.get(4),
                    description: row.get(5),
                    created_at: row.get(6),
                    updated_at: row.get(7),
                };
                origins.push(origin);
            }
            Ok(warp::reply::json(&origins))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(&json!({"error": "Failed to fetch origins"})))
        }
    }
}

pub async fn create_origin(
    auth: AuthContext,
    create_origin: CreateOrigin,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    // Validate input
    create_origin.validate().map_err(warp::reject::custom)?;

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
            "INSERT INTO origins (id, user_id, name, country, region, description, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
         RETURNING id, user_id, name, country, region, description, created_at, updated_at",
            &[
                &id,
                &auth.user_id,
                &create_origin.name,
                &create_origin.country,
                &create_origin.region,
                &create_origin.description,
                &now,
                &now,
            ],
        )
        .await
    {
        Ok(row) => {
            let origin = Origin {
                id: row.get(0),
                user_id: row.get(1),
                name: row.get(2),
                country: row.get(3),
                region: row.get(4),
                description: row.get(5),
                created_at: row.get(6),
                updated_at: row.get(7),
            };
            Ok(warp::reply::json(&origin))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(
                &json!({"error": "Failed to create origin"}),
            ))
        }
    }
}

pub async fn update_origin(
    id: Uuid,
    auth: AuthContext,
    update_origin: UpdateOrigin,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    // Validate input
    update_origin.validate().map_err(warp::reject::custom)?;

    let now = Utc::now();

    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    match db
        .query_opt(
            "UPDATE origins SET 
         name = COALESCE($2, name),
         country = COALESCE($3, country),
         region = COALESCE($4, region),
         description = COALESCE($5, description),
         updated_at = $6
         WHERE id = $1 AND user_id = $7
         RETURNING id, user_id, name, country, region, description, created_at, updated_at",
            &[
                &id,
                &update_origin.name,
                &update_origin.country,
                &update_origin.region,
                &update_origin.description,
                &now,
                &auth.user_id,
            ],
        )
        .await
    {
        Ok(Some(row)) => {
            let origin = Origin {
                id: row.get(0),
                user_id: row.get(1),
                name: row.get(2),
                country: row.get(3),
                region: row.get(4),
                description: row.get(5),
                created_at: row.get(6),
                updated_at: row.get(7),
            };
            Ok(warp::reply::json(&origin))
        }
        Ok(None) => Ok(warp::reply::json(
            &json!({"error": "Origin not found or unauthorized"}),
        )),
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(
                &json!({"error": "Failed to update origin"}),
            ))
        }
    }
}

pub async fn delete_origin(
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
            "DELETE FROM origins WHERE id = $1 AND user_id = $2",
            &[&id, &auth.user_id],
        )
        .await
    {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok(warp::reply::json(
                    &json!({"message": "Origin deleted successfully"}),
                ))
            } else {
                Ok(warp::reply::json(
                    &json!({"error": "Origin not found or unauthorized"}),
                ))
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(
                &json!({"error": "Failed to delete origin"}),
            ))
        }
    }
}
