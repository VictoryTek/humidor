use chrono::Utc;
use serde_json::json;
use uuid::Uuid;
use warp::{Rejection, Reply};

use crate::{errors::AppError, middleware::AuthContext, models::*, validation::Validate, DbPool};

pub async fn get_ring_gauges(auth: AuthContext, pool: DbPool) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    match db.query(
        "SELECT id, user_id, gauge, description, common_names, created_at, updated_at FROM ring_gauges WHERE user_id = $1 ORDER BY gauge ASC",
        &[&auth.user_id]
    ).await {
        Ok(rows) => {
            let mut ring_gauges = Vec::new();
            for row in rows {
                let ring_gauge = RingGauge {
                    id: row.get(0),
                    user_id: row.get(1),
                    gauge: row.get(2),
                    description: row.get(3),
                    common_names: row.get(4),
                    created_at: row.get(5),
                    updated_at: row.get(6),
                };
                ring_gauges.push(ring_gauge);
            }
            Ok(warp::reply::json(&ring_gauges))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(&json!({"error": "Failed to fetch ring gauges"})))
        }
    }
}

pub async fn create_ring_gauge(
    auth: AuthContext,
    create_ring_gauge: CreateRingGauge,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    // Validate input
    create_ring_gauge.validate().map_err(warp::reject::custom)?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    match db.query_one(
        "INSERT INTO ring_gauges (id, user_id, gauge, description, common_names, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7) 
         RETURNING id, user_id, gauge, description, common_names, created_at, updated_at",
        &[&id, &auth.user_id, &create_ring_gauge.gauge, &create_ring_gauge.description, &create_ring_gauge.common_names, &now, &now]
    ).await {
        Ok(row) => {
            let ring_gauge = RingGauge {
                id: row.get(0),
                user_id: row.get(1),
                gauge: row.get(2),
                description: row.get(3),
                common_names: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
            };
            Ok(warp::reply::json(&ring_gauge))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(&json!({"error": "Failed to create ring gauge"})))
        }
    }
}

pub async fn update_ring_gauge(
    id: Uuid,
    auth: AuthContext,
    update_ring_gauge: UpdateRingGauge,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    // Validate input
    update_ring_gauge.validate().map_err(warp::reject::custom)?;

    let now = Utc::now();

    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    match db
        .query_opt(
            "UPDATE ring_gauges SET 
         gauge = COALESCE($2, gauge),
         description = COALESCE($3, description),
         common_names = COALESCE($4, common_names),
         updated_at = $5
         WHERE id = $1 AND user_id = $6
         RETURNING id, user_id, gauge, description, common_names, created_at, updated_at",
            &[
                &id,
                &update_ring_gauge.gauge,
                &update_ring_gauge.description,
                &update_ring_gauge.common_names,
                &now,
                &auth.user_id,
            ],
        )
        .await
    {
        Ok(Some(row)) => {
            let ring_gauge = RingGauge {
                id: row.get(0),
                user_id: row.get(1),
                gauge: row.get(2),
                description: row.get(3),
                common_names: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
            };
            Ok(warp::reply::json(&ring_gauge))
        }
        Ok(None) => Ok(warp::reply::json(
            &json!({"error": "Ring gauge not found or unauthorized"}),
        )),
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(
                &json!({"error": "Failed to update ring gauge"}),
            ))
        }
    }
}

pub async fn delete_ring_gauge(
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
            "DELETE FROM ring_gauges WHERE id = $1 AND user_id = $2",
            &[&id, &auth.user_id],
        )
        .await
    {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok(warp::reply::json(
                    &json!({"message": "Ring gauge deleted successfully"}),
                ))
            } else {
                Ok(warp::reply::json(
                    &json!({"error": "Ring gauge not found or unauthorized"}),
                ))
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(
                &json!({"error": "Failed to delete ring gauge"}),
            ))
        }
    }
}
