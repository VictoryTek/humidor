use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{Reply, Rejection, reply::json};

use crate::{DbPool, middleware::auth::AuthContext};

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
pub async fn get_favorites(
    auth: AuthContext,
    db: DbPool
) -> Result<impl Reply, Rejection> {
    let rows = db.query(
        "SELECT f.id, f.user_id, f.cigar_id, f.created_at,
                c.id as c_id, c.humidor_id, c.brand_id, c.name, c.size_id, c.strength_id, 
                c.origin_id, c.wrapper, c.binder, c.filler, c.price, c.purchase_date, 
                c.notes, c.quantity, c.ring_gauge_id, c.length, c.image_url, 
                c.created_at as c_created_at, c.updated_at as c_updated_at
         FROM favorites f
         JOIN cigars c ON f.cigar_id = c.id
         WHERE f.user_id = $1
         ORDER BY f.created_at DESC",
        &[&auth.user_id]
    ).await.map_err(|e| {
        eprintln!("Database error getting favorites: {}", e);
        warp::reject::reject()
    })?;

    let favorites: Vec<serde_json::Value> = rows.iter().map(|row| {
        serde_json::json!({
            "id": row.get::<_, Uuid>(0),
            "user_id": row.get::<_, Uuid>(1),
            "cigar_id": row.get::<_, Uuid>(2),
            "created_at": row.get::<_, chrono::DateTime<Utc>>(3),
            "cigar": {
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
            }
        })
    }).collect();

    Ok(json(&favorites))
}

// Add a cigar to favorites
pub async fn add_favorite(
    request: AddFavoriteRequest,
    auth: AuthContext,
    db: DbPool
) -> Result<impl Reply, Rejection> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    
    match db.query_one(
        "INSERT INTO favorites (id, user_id, cigar_id, created_at)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id, cigar_id) DO NOTHING
         RETURNING id, user_id, cigar_id, created_at",
        &[&id, &auth.user_id, &request.cigar_id, &now]
    ).await {
        Ok(row) => {
            let favorite = FavoriteResponse {
                id: row.get(0),
                user_id: row.get(1),
                cigar_id: row.get(2),
                created_at: row.get(3),
            };
            Ok(warp::reply::with_status(json(&favorite), warp::http::StatusCode::CREATED))
        },
        Err(e) => {
            eprintln!("Database error adding favorite: {}", e);
            Err(warp::reject::reject())
        }
    }
}

// Remove a cigar from favorites
pub async fn remove_favorite(
    cigar_id: Uuid,
    auth: AuthContext,
    db: DbPool
) -> Result<impl Reply, Rejection> {
    match db.execute(
        "DELETE FROM favorites WHERE user_id = $1 AND cigar_id = $2",
        &[&auth.user_id, &cigar_id]
    ).await {
        Ok(_) => Ok(warp::reply::with_status(json(&serde_json::json!({"message": "Favorite removed"})), warp::http::StatusCode::OK)),
        Err(e) => {
            eprintln!("Database error removing favorite: {}", e);
            Err(warp::reject::reject())
        }
    }
}

// Check if a cigar is favorited
pub async fn is_favorite(
    cigar_id: Uuid,
    auth: AuthContext,
    db: DbPool
) -> Result<impl Reply, Rejection> {
    match db.query_opt(
        "SELECT id FROM favorites WHERE user_id = $1 AND cigar_id = $2",
        &[&auth.user_id, &cigar_id]
    ).await {
        Ok(row) => {
            let is_favorite = row.is_some();
            Ok(json(&serde_json::json!({"is_favorite": is_favorite})))
        },
        Err(e) => {
            eprintln!("Database error checking favorite: {}", e);
            Err(warp::reject::reject())
        }
    }
}
