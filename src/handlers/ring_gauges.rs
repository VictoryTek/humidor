use chrono::Utc;
use serde_json::json;
use uuid::Uuid;
use warp::{Rejection, Reply};

use crate::{errors::AppError, models::*, validation::Validate, DbPool};

pub async fn get_ring_gauges(pool: DbPool) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    match db.query(
        "SELECT id, gauge, description, common_names, created_at, updated_at FROM ring_gauges ORDER BY gauge ASC",
        &[]
    ).await {
        Ok(rows) => {
            let mut ring_gauges = Vec::new();
            for row in rows {
                let ring_gauge = RingGauge {
                    id: row.get(0),
                    gauge: row.get(1),
                    description: row.get(2),
                    common_names: row.get(3),
                    created_at: row.get(4),
                    updated_at: row.get(5),
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
        "INSERT INTO ring_gauges (id, gauge, description, common_names, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6) 
         RETURNING id, gauge, description, common_names, created_at, updated_at",
        &[&id, &create_ring_gauge.gauge, &create_ring_gauge.description, &create_ring_gauge.common_names, &now, &now]
    ).await {
        Ok(row) => {
            let ring_gauge = RingGauge {
                id: row.get(0),
                gauge: row.get(1),
                description: row.get(2),
                common_names: row.get(3),
                created_at: row.get(4),
                updated_at: row.get(5),
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
        .query_one(
            "UPDATE ring_gauges SET 
         gauge = COALESCE($2, gauge),
         description = COALESCE($3, description),
         common_names = COALESCE($4, common_names),
         updated_at = $5
         WHERE id = $1
         RETURNING id, gauge, description, common_names, created_at, updated_at",
            &[
                &id,
                &update_ring_gauge.gauge,
                &update_ring_gauge.description,
                &update_ring_gauge.common_names,
                &now,
            ],
        )
        .await
    {
        Ok(row) => {
            let ring_gauge = RingGauge {
                id: row.get(0),
                gauge: row.get(1),
                description: row.get(2),
                common_names: row.get(3),
                created_at: row.get(4),
                updated_at: row.get(5),
            };
            Ok(warp::reply::json(&ring_gauge))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(
                &json!({"error": "Failed to update ring gauge"}),
            ))
        }
    }
}

pub async fn delete_ring_gauge(id: Uuid, pool: DbPool) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    match db
        .execute("DELETE FROM ring_gauges WHERE id = $1", &[&id])
        .await
    {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok(warp::reply::json(
                    &json!({"message": "Ring gauge deleted successfully"}),
                ))
            } else {
                Ok(warp::reply::json(&json!({"error": "Ring gauge not found"})))
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
