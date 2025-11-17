use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{Rejection, Reply, reply::json};

use crate::{DbPool, errors::AppError, middleware::auth::AuthContext};

#[derive(Debug, Serialize)]
pub struct WishListResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub cigar_id: Uuid,
    pub notes: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddWishListRequest {
    pub cigar_id: Uuid,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWishListNotesRequest {
    pub notes: Option<String>,
}

// Get all wish list items for the current user
pub async fn get_wish_list(auth: AuthContext, pool: DbPool) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    let rows = db
        .query(
            "SELECT w.id, w.user_id, w.cigar_id, w.notes, w.created_at,
                c.id as c_id, c.humidor_id, c.brand_id, c.name, c.size_id, c.strength_id,
                c.origin_id, c.wrapper, c.binder, c.filler, c.price, c.purchase_date,
                c.notes as c_notes, c.quantity, c.ring_gauge_id, c.length, c.image_url,
                c.created_at as c_created_at, c.updated_at as c_updated_at, c.is_active
         FROM wish_list w
         LEFT JOIN cigars c ON w.cigar_id = c.id
         WHERE w.user_id = $1
         ORDER BY w.created_at DESC",
            &[&auth.user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Database error getting wish list");
            warp::reject::reject()
        })?;

    let wish_list: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            let cigar_exists: Option<Uuid> = row.get(5); // c.id will be null if cigar doesn't exist

            serde_json::json!({
                "id": row.get::<_, Uuid>(0),
                "user_id": row.get::<_, Uuid>(1),
                "cigar_id": row.get::<_, Uuid>(2),
                "notes": row.get::<_, Option<String>>(3),
                "created_at": row.get::<_, chrono::DateTime<Utc>>(4),
                "cigar": if cigar_exists.is_some() {
                    let is_active: bool = row.get(24);
                    serde_json::json!({
                        "id": row.get::<_, Uuid>(5),
                        "humidor_id": row.get::<_, Option<Uuid>>(6),
                        "brand_id": row.get::<_, Option<Uuid>>(7),
                        "name": row.get::<_, String>(8),
                        "size_id": row.get::<_, Option<Uuid>>(9),
                        "strength_id": row.get::<_, Option<Uuid>>(10),
                        "origin_id": row.get::<_, Option<Uuid>>(11),
                        "wrapper": row.get::<_, Option<String>>(12),
                        "binder": row.get::<_, Option<String>>(13),
                        "filler": row.get::<_, Option<String>>(14),
                        "price": row.get::<_, Option<f64>>(15),
                        "purchase_date": row.get::<_, Option<chrono::DateTime<Utc>>>(16),
                        "notes": row.get::<_, Option<String>>(17),
                        "quantity": row.get::<_, i32>(18),
                        "ring_gauge_id": row.get::<_, Option<Uuid>>(19),
                        "length": row.get::<_, Option<f64>>(20),
                        "image_url": row.get::<_, Option<String>>(21),
                        "created_at": row.get::<_, chrono::DateTime<Utc>>(22),
                        "updated_at": row.get::<_, chrono::DateTime<Utc>>(23),
                        "out_of_stock": !is_active
                    })
                } else {
                    serde_json::json!(null)
                }
            })
        })
        .collect();

    Ok(json(&wish_list))
}

// Add a cigar to wish list
pub async fn add_to_wish_list(
    request: AddWishListRequest,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    tracing::debug!("add_to_wish_list handler called");
    tracing::debug!(user_id = %auth.user_id, "Processing wish list request");
    tracing::debug!(cigar_id = %request.cigar_id, "Adding cigar to wish list");

    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    // Parse cigar_id from string to ensure it's a proper Uuid type
    let cigar_id = Uuid::parse_str(&request.cigar_id.to_string()).map_err(|e| {
        tracing::error!(error = %e, "Invalid cigar_id format");
        warp::reject::custom(AppError::ValidationError(
            "Invalid cigar ID format".to_string(),
        ))
    })?;

    // Check if cigar exists
    let cigar_exists = db
        .query_opt("SELECT id FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Database error checking cigar");
            warp::reject::reject()
        })?;

    if cigar_exists.is_none() {
        tracing::warn!(cigar_id = %cigar_id, "Cigar not found");
        return Err(warp::reject::custom(AppError::NotFound(
            "Cigar not found".to_string(),
        )));
    }

    // Insert the wish list item with notes
    let result = db
        .query_opt(
            "INSERT INTO wish_list (id, user_id, cigar_id, notes, created_at)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (user_id, cigar_id) DO NOTHING
         RETURNING id, user_id, cigar_id, notes, created_at",
            &[
                &id,
                &auth.user_id,
                &cigar_id,
                &request.notes.as_deref(),
                &now,
            ],
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Database error adding to wish list");
            warp::reject::reject()
        })?;

    // If insert succeeded, return the new item
    if let Some(row) = result {
        let wish_list_item = WishListResponse {
            id: row.get(0),
            user_id: row.get(1),
            cigar_id: row.get(2),
            notes: row.get(3),
            created_at: row.get(4),
        };
        Ok(warp::reply::with_status(
            json(&wish_list_item),
            warp::http::StatusCode::CREATED,
        ))
    } else {
        // Item already exists (conflict), fetch and return it
        match db
            .query_one(
                "SELECT id, user_id, cigar_id, notes, created_at
             FROM wish_list
             WHERE user_id = $1 AND cigar_id = $2",
                &[&auth.user_id, &request.cigar_id],
            )
            .await
        {
            Ok(row) => {
                let wish_list_item = WishListResponse {
                    id: row.get(0),
                    user_id: row.get(1),
                    cigar_id: row.get(2),
                    notes: row.get(3),
                    created_at: row.get(4),
                };
                Ok(warp::reply::with_status(
                    json(&wish_list_item),
                    warp::http::StatusCode::OK,
                ))
            }
            Err(e) => {
                tracing::error!(error = %e, "Database error fetching existing wish list item");
                Err(warp::reject::reject())
            }
        }
    }
}

// Remove a cigar from wish list
pub async fn remove_from_wish_list(
    cigar_id: Uuid,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    let rows_deleted = db
        .execute(
            "DELETE FROM wish_list WHERE user_id = $1 AND cigar_id = $2",
            &[&auth.user_id, &cigar_id],
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Database error removing from wish list");
            warp::reject::reject()
        })?;

    if rows_deleted == 0 {
        return Err(warp::reject::custom(AppError::NotFound(
            "Wish list item not found".to_string(),
        )));
    }

    Ok(warp::reply::with_status(
        json(&serde_json::json!({"message": "Removed from wish list"})),
        warp::http::StatusCode::OK,
    ))
}

// Check if a cigar is in wish list
pub async fn check_wish_list(
    cigar_id: Uuid,
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
        .query_opt(
            "SELECT id FROM wish_list WHERE user_id = $1 AND cigar_id = $2",
            &[&auth.user_id, &cigar_id],
        )
        .await
    {
        Ok(row) => {
            let in_wish_list = row.is_some();
            Ok(json(&serde_json::json!({"in_wish_list": in_wish_list})))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error checking wish list");
            Err(warp::reject::reject())
        }
    }
}

// Update wish list notes
pub async fn update_wish_list_notes(
    cigar_id: Uuid,
    request: UpdateWishListNotesRequest,
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
        .query_opt(
            "UPDATE wish_list SET notes = $1 
             WHERE user_id = $2 AND cigar_id = $3
             RETURNING id, user_id, cigar_id, notes, created_at",
            &[&request.notes, &auth.user_id, &cigar_id],
        )
        .await
    {
        Ok(Some(row)) => {
            let wish_list_item = WishListResponse {
                id: row.get(0),
                user_id: row.get(1),
                cigar_id: row.get(2),
                notes: row.get(3),
                created_at: row.get(4),
            };
            Ok(json(&wish_list_item))
        }
        Ok(None) => Err(warp::reject::custom(AppError::NotFound(
            "Wish list item not found".to_string(),
        ))),
        Err(e) => {
            tracing::error!(error = %e, "Database error updating wish list notes");
            Err(warp::reject::reject())
        }
    }
}
