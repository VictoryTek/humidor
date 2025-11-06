use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{reply::json, Rejection, Reply};

use crate::{DbPool, middleware::auth::AuthContext, errors::AppError};

#[derive(Debug, Serialize)]
pub struct FavoriteResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub cigar_id: Uuid,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddFavoriteRequest {
    pub cigar_id: Uuid,
}

// Get all favorites for the current user
pub async fn get_favorites(auth: AuthContext, pool: DbPool) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError("Database connection failed".to_string()))
    })?;
    
    let rows = db
        .query(
            "SELECT f.id, f.user_id, f.cigar_id, f.created_at,
                c.id as c_id, c.humidor_id, c.brand_id, c.name, c.size_id, c.strength_id,
                c.origin_id, c.wrapper, c.binder, c.filler, c.price, c.purchase_date,
                c.notes, c.quantity, c.ring_gauge_id, c.length, c.image_url,
                c.created_at as c_created_at, c.updated_at as c_updated_at, c.is_active,
                f.snapshot_name, f.snapshot_brand_id, f.snapshot_size_id,
                f.snapshot_strength_id, f.snapshot_origin_id,
                f.snapshot_ring_gauge_id, f.snapshot_image_url
         FROM favorites f
         LEFT JOIN cigars c ON f.cigar_id = c.id
         WHERE f.user_id = $1
         ORDER BY f.created_at DESC",
            &[&auth.user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Database error getting favorites");
            warp::reject::reject()
        })?;

    let favorites: Vec<serde_json::Value> = rows.iter().map(|row| {
        let cigar_id: Option<Uuid> = row.get(2);
        let cigar_exists: Option<Uuid> = row.get(4); // c.id will be null if cigar doesn't exist

        serde_json::json!({
            "id": row.get::<_, Uuid>(0),
            "user_id": row.get::<_, Uuid>(1),
            "cigar_id": cigar_id,
            "created_at": row.get::<_, chrono::DateTime<Utc>>(3),
            "cigar": if cigar_exists.is_some() {
                // Cigar still exists, return live data
                let is_active: bool = row.get(23);
                serde_json::json!({
                    "id": row.get::<_, Uuid>(4),
                    "humidor_id": row.get::<_, Option<Uuid>>(5),
                    "brand_id": row.get::<_, Option<Uuid>>(6),
                    "name": row.get::<_, String>(7),
                    "size_id": row.get::<_, Option<Uuid>>(8),
                    "strength_id": row.get::<_, Option<Uuid>>(9),
                    "origin_id": row.get::<_, Option<Uuid>>(10),
                    "wrapper": row.get::<_, Option<String>>(11),
                    "binder": row.get::<_, Option<String>>(12),
                    "filler": row.get::<_, Option<String>>(13),
                    "price": row.get::<_, Option<f64>>(14),
                    "purchase_date": row.get::<_, Option<chrono::DateTime<Utc>>>(15),
                    "notes": row.get::<_, Option<String>>(16),
                    "quantity": row.get::<_, i32>(17),
                    "ring_gauge_id": row.get::<_, Option<Uuid>>(18),
                    "length": row.get::<_, Option<f64>>(19),
                    "image_url": row.get::<_, Option<String>>(20),
                    "created_at": row.get::<_, chrono::DateTime<Utc>>(21),
                    "updated_at": row.get::<_, chrono::DateTime<Utc>>(22),
                    "out_of_stock": !is_active
                })
            } else {
                // Cigar deleted, return snapshot data
                serde_json::json!({
                    "id": cigar_id,
                    "humidor_id": serde_json::Value::Null,
                    "brand_id": row.get::<_, Option<Uuid>>(25),
                    "name": row.get::<_, Option<String>>(24).unwrap_or_else(|| "Unknown Cigar".to_string()),
                    "size_id": row.get::<_, Option<Uuid>>(26),
                    "strength_id": row.get::<_, Option<Uuid>>(27),
                    "origin_id": row.get::<_, Option<Uuid>>(28),
                    "wrapper": serde_json::Value::Null,
                    "binder": serde_json::Value::Null,
                    "filler": serde_json::Value::Null,
                    "price": serde_json::Value::Null,
                    "purchase_date": serde_json::Value::Null,
                    "notes": serde_json::Value::Null,
                    "quantity": 0,
                    "ring_gauge_id": row.get::<_, Option<Uuid>>(29),
                    "length": serde_json::Value::Null,
                    "image_url": row.get::<_, Option<String>>(30),
                    "created_at": serde_json::Value::Null,
                    "updated_at": serde_json::Value::Null,
                    "out_of_stock": true
                })
            }
        })
    }).collect();

    Ok(json(&favorites))
}

// Add a cigar to favorites
pub async fn add_favorite(
    request: AddFavoriteRequest,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError("Database connection failed".to_string()))
    })?;
    
    let id = Uuid::new_v4();
    let now = Utc::now();

    // First, get the cigar data to create a snapshot
    let cigar = db
        .query_opt(
            "SELECT name, brand_id, size_id, strength_id, origin_id, ring_gauge_id, image_url
         FROM cigars WHERE id = $1",
            &[&request.cigar_id],
        )
        .await
        .map_err(|e| {
            tracing::error!(
                cigar_id = %request.cigar_id,
                error = %e,
                "Database error fetching cigar for snapshot"
            );
            warp::reject::reject()
        })?;

    let cigar = match cigar {
        Some(row) => row,
        None => {
            tracing::warn!(
                cigar_id = %request.cigar_id,
                "Attempted to favorite non-existent cigar"
            );
            return Err(warp::reject::reject());
        }
    };
    
    let snapshot_name: String = cigar.get(0);
    let snapshot_brand_id: Option<Uuid> = cigar.get(1);
    let snapshot_size_id: Option<Uuid> = cigar.get(2);
    let snapshot_strength_id: Option<Uuid> = cigar.get(3);
    let snapshot_origin_id: Option<Uuid> = cigar.get(4);
    let snapshot_ring_gauge_id: Option<Uuid> = cigar.get(5);
    let snapshot_image_url: Option<String> = cigar.get(6);

    match db
        .query_one(
            "INSERT INTO favorites (id, user_id, cigar_id, created_at,
                                snapshot_name, snapshot_brand_id, snapshot_size_id,
                                snapshot_strength_id, snapshot_origin_id,
                                snapshot_ring_gauge_id, snapshot_image_url)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         ON CONFLICT (user_id, cigar_id) DO NOTHING
         RETURNING id, user_id, cigar_id, created_at",
            &[
                &id,
                &auth.user_id,
                &request.cigar_id,
                &now,
                &snapshot_name,
                &snapshot_brand_id,
                &snapshot_size_id,
                &snapshot_strength_id,
                &snapshot_origin_id,
                &snapshot_ring_gauge_id,
                &snapshot_image_url,
            ],
        )
        .await
    {
        Ok(row) => {
            let favorite = FavoriteResponse {
                id: row.get(0),
                user_id: row.get(1),
                cigar_id: row.get(2),
                created_at: row.get(3),
            };
            Ok(warp::reply::with_status(
                json(&favorite),
                warp::http::StatusCode::CREATED,
            ))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error adding favorite");
            Err(warp::reject::reject())
        }
    }
}

// Remove a cigar from favorites
// Accepts either cigar_id or favorite_id (for deleted cigars)
pub async fn remove_favorite(
    id: Uuid,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError("Database connection failed".to_string()))
    })?;
    
    // Try to delete by cigar_id first
    let rows_deleted = db
        .execute(
            "DELETE FROM favorites WHERE user_id = $1 AND cigar_id = $2",
            &[&auth.user_id, &id],
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Database error removing favorite by cigar_id");
            warp::reject::reject()
        })?;

    // If no rows deleted, try deleting by favorite id (for orphaned favorites)
    if rows_deleted == 0 {
        db.execute(
            "DELETE FROM favorites WHERE user_id = $1 AND id = $2",
            &[&auth.user_id, &id],
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Database error removing favorite by id");
            warp::reject::reject()
        })?;
    }

    Ok(warp::reply::with_status(
        json(&serde_json::json!({"message": "Favorite removed"})),
        warp::http::StatusCode::OK,
    ))
}

// Check if a cigar is favorited
pub async fn is_favorite(
    cigar_id: Uuid,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError("Database connection failed".to_string()))
    })?;
    
    match db
        .query_opt(
            "SELECT id FROM favorites WHERE user_id = $1 AND cigar_id = $2",
            &[&auth.user_id, &cigar_id],
        )
        .await
    {
        Ok(row) => {
            let is_favorite = row.is_some();
            Ok(json(&serde_json::json!({"is_favorite": is_favorite})))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error checking favorite");
            Err(warp::reject::reject())
        }
    }
}
