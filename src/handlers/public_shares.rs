use crate::errors::AppError;
use crate::middleware::auth::AuthContext;
use crate::models::{
    CreatePublicShareRequest, PublicCigarResponse, PublicHumidorResponse, PublicShareResponse,
    PublicUserInfo,
};
use deadpool_postgres::Pool;
use std::env;
use uuid::Uuid;
use warp::{Rejection, Reply, reject, reply};

use super::humidor_shares::is_humidor_owner;

/// Create or update public share for a humidor
/// POST /api/v1/humidors/:id/public-share
pub async fn create_public_share(
    humidor_id: Uuid,
    auth: AuthContext,
    request: CreatePublicShareRequest,
    pool: Pool,
) -> Result<impl Reply, Rejection> {
    tracing::info!(
        "User {} creating public share for humidor {}",
        auth.user_id,
        humidor_id
    );

    // Verify user is the owner of the humidor
    if !is_humidor_owner(&pool, &auth.user_id, &humidor_id)
        .await
        .map_err(reject::custom)?
    {
        tracing::warn!(
            "User {} attempted to create public share for humidor {} they don't own",
            auth.user_id,
            humidor_id
        );
        return Err(reject::custom(AppError::Forbidden(
            "You do not have permission to create a public share for this humidor".to_string(),
        )));
    }

    let client = pool.get().await.map_err(|e| {
        tracing::error!("Failed to get database connection: {}", e);
        reject::custom(AppError::DatabaseError(
            "Failed to connect to database".to_string(),
        ))
    })?;

    let token_id = Uuid::new_v4();

    // Determine expiration: never_expires = NULL, custom date, or default 30 days
    let expires_at = if request.never_expires {
        None
    } else if let Some(custom_date) = request.expires_at {
        Some(custom_date)
    } else {
        // Default: 30 days from now
        Some(chrono::Utc::now() + chrono::Duration::days(30))
    };

    // Insert new public share (allows multiple shares per humidor)
    let row = client
        .query_one(
            "INSERT INTO public_humidor_shares (id, humidor_id, created_by_user_id, expires_at, include_favorites, include_wish_list, label, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
             RETURNING created_at",
            &[&token_id, &humidor_id, &auth.user_id, &expires_at, &request.include_favorites, &request.include_wish_list, &request.label],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create public share: {}", e);
            reject::custom(AppError::DatabaseError(
                "Failed to create public share".to_string(),
            ))
        })?;

    let created_at = row.get(0);

    // Get domain from environment or use localhost for development
    let domain = env::var("DOMAIN").unwrap_or_else(|_| "localhost:9898".to_string());
    let protocol = if domain.contains("localhost") {
        "http"
    } else {
        "https"
    };
    let share_url = format!("{}://{}/shared/humidors/{}", protocol, domain, token_id);

    tracing::info!(
        "Successfully created public share for humidor {} with token {} (expires: {:?})",
        humidor_id,
        token_id,
        expires_at
    );

    Ok(reply::with_status(
        reply::json(&PublicShareResponse {
            token_id,
            share_url,
            expires_at,
            created_at,
            include_favorites: request.include_favorites,
            include_wish_list: request.include_wish_list,
            label: request.label,
        }),
        warp::http::StatusCode::CREATED,
    ))
}

/// Get all public shares for a humidor
/// GET /api/v1/humidors/:id/public-shares
pub async fn get_public_shares(
    humidor_id: Uuid,
    auth: AuthContext,
    pool: Pool,
) -> Result<impl Reply, Rejection> {
    tracing::debug!(
        "User {} fetching public shares for humidor {}",
        auth.user_id,
        humidor_id
    );

    // Verify user is the owner (or has full permission to manage shares)
    if !is_humidor_owner(&pool, &auth.user_id, &humidor_id)
        .await
        .map_err(reject::custom)?
    {
        return Err(reject::custom(AppError::Forbidden(
            "You do not have permission to view public share settings".to_string(),
        )));
    }

    let client = pool.get().await.map_err(|e| {
        tracing::error!("Failed to get database connection: {}", e);
        reject::custom(AppError::DatabaseError(
            "Failed to connect to database".to_string(),
        ))
    })?;

    // Fetch all active public shares (check expiration if set)
    let rows = client
        .query(
            "SELECT id, expires_at, created_at, include_favorites, include_wish_list, label 
             FROM public_humidor_shares 
             WHERE humidor_id = $1 AND (expires_at IS NULL OR expires_at > NOW())
             ORDER BY created_at DESC",
            &[&humidor_id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch public shares: {}", e);
            reject::custom(AppError::DatabaseError(
                "Failed to fetch public shares".to_string(),
            ))
        })?;

    let domain = env::var("DOMAIN").unwrap_or_else(|_| "localhost:9898".to_string());
    let protocol = if domain.contains("localhost") {
        "http"
    } else {
        "https"
    };

    let shares: Vec<PublicShareResponse> = rows
        .iter()
        .map(|row| {
            let token_id: Uuid = row.get(0);
            let expires_at: Option<chrono::DateTime<chrono::Utc>> = row.get(1);
            let created_at = row.get(2);
            let include_favorites: bool = row.get(3);
            let include_wish_list: bool = row.get(4);
            let label: Option<String> = row.get(5);

            let share_url = format!("{}://{}/shared/humidors/{}", protocol, domain, token_id);

            PublicShareResponse {
                token_id,
                share_url,
                expires_at,
                created_at,
                include_favorites,
                include_wish_list,
                label,
            }
        })
        .collect();

    Ok(reply::json(&shares))
}

/// Get current public share information for a humidor (kept for backward compatibility)
/// GET /api/v1/humidors/:id/public-share
pub async fn get_public_share(
    humidor_id: Uuid,
    auth: AuthContext,
    pool: Pool,
) -> Result<impl Reply, Rejection> {
    tracing::debug!(
        "User {} fetching public share for humidor {}",
        auth.user_id,
        humidor_id
    );

    // Verify user is the owner (or has full permission to manage shares)
    if !is_humidor_owner(&pool, &auth.user_id, &humidor_id)
        .await
        .map_err(reject::custom)?
    {
        return Err(reject::custom(AppError::Forbidden(
            "You do not have permission to view public share settings".to_string(),
        )));
    }

    let client = pool.get().await.map_err(|e| {
        tracing::error!("Failed to get database connection: {}", e);
        reject::custom(AppError::DatabaseError(
            "Failed to connect to database".to_string(),
        ))
    })?;

    // Fetch most recent active public share (check expiration if set)
    let row = client
        .query_opt(
            "SELECT id, expires_at, created_at, include_favorites, include_wish_list, label 
             FROM public_humidor_shares 
             WHERE humidor_id = $1 AND (expires_at IS NULL OR expires_at > NOW())
             ORDER BY created_at DESC
             LIMIT 1",
            &[&humidor_id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch public share: {}", e);
            reject::custom(AppError::DatabaseError(
                "Failed to fetch public share".to_string(),
            ))
        })?;

    match row {
        Some(row) => {
            let token_id: Uuid = row.get(0);
            let expires_at: Option<chrono::DateTime<chrono::Utc>> = row.get(1);
            let created_at = row.get(2);
            let include_favorites: bool = row.get(3);
            let include_wish_list: bool = row.get(4);
            let label: Option<String> = row.get(5);

            let domain = env::var("DOMAIN").unwrap_or_else(|_| "localhost:9898".to_string());
            let protocol = if domain.contains("localhost") {
                "http"
            } else {
                "https"
            };
            let share_url = format!("{}://{}/shared/humidors/{}", protocol, domain, token_id);

            Ok(reply::json(&PublicShareResponse {
                token_id,
                share_url,
                expires_at,
                created_at,
                include_favorites,
                include_wish_list,
                label,
            }))
        }
        None => {
            // Check if expired share exists and clean it up
            let _ = client
                .execute(
                    "DELETE FROM public_humidor_shares 
                     WHERE humidor_id = $1 AND expires_at <= NOW()",
                    &[&humidor_id],
                )
                .await;

            Err(reject::custom(AppError::NotFound(
                "No active public share found".to_string(),
            )))
        }
    }
}

/// Revoke public share for a humidor
/// DELETE /api/v1/humidors/:id/public-share
pub async fn revoke_public_share(
    humidor_id: Uuid,
    auth: AuthContext,
    pool: Pool,
) -> Result<impl Reply, Rejection> {
    tracing::info!(
        "User {} revoking all public shares for humidor {}",
        auth.user_id,
        humidor_id
    );

    // Verify user is the owner
    if !is_humidor_owner(&pool, &auth.user_id, &humidor_id)
        .await
        .map_err(reject::custom)?
    {
        return Err(reject::custom(AppError::Forbidden(
            "You do not have permission to revoke the public shares".to_string(),
        )));
    }

    let client = pool.get().await.map_err(|e| {
        tracing::error!("Failed to get database connection: {}", e);
        reject::custom(AppError::DatabaseError(
            "Failed to connect to database".to_string(),
        ))
    })?;

    let rows_affected = client
        .execute(
            "DELETE FROM public_humidor_shares WHERE humidor_id = $1",
            &[&humidor_id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete public shares: {}", e);
            reject::custom(AppError::DatabaseError(
                "Failed to revoke public shares".to_string(),
            ))
        })?;

    if rows_affected == 0 {
        return Err(reject::custom(AppError::NotFound(
            "No public shares found to revoke".to_string(),
        )));
    }

    tracing::info!(
        "Successfully revoked {} public share(s) for humidor {}",
        rows_affected,
        humidor_id
    );

    Ok(reply::with_status(
        reply::json(&serde_json::json!({
            "message": format!("Revoked {} public share(s) successfully", rows_affected)
        })),
        warp::http::StatusCode::OK,
    ))
}

/// Delete a specific public share by token ID
/// DELETE /api/v1/humidors/:id/public-shares/:token_id
pub async fn delete_public_share(
    humidor_id: Uuid,
    token_id: Uuid,
    auth: AuthContext,
    pool: Pool,
) -> Result<impl Reply, Rejection> {
    tracing::info!(
        "User {} deleting public share {} for humidor {}",
        auth.user_id,
        token_id,
        humidor_id
    );

    // Verify user is the owner
    if !is_humidor_owner(&pool, &auth.user_id, &humidor_id)
        .await
        .map_err(reject::custom)?
    {
        return Err(reject::custom(AppError::Forbidden(
            "You do not have permission to delete this public share".to_string(),
        )));
    }

    let client = pool.get().await.map_err(|e| {
        tracing::error!("Failed to get database connection: {}", e);
        reject::custom(AppError::DatabaseError(
            "Failed to connect to database".to_string(),
        ))
    })?;

    let rows_affected = client
        .execute(
            "DELETE FROM public_humidor_shares WHERE id = $1 AND humidor_id = $2",
            &[&token_id, &humidor_id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete public share: {}", e);
            reject::custom(AppError::DatabaseError(
                "Failed to delete public share".to_string(),
            ))
        })?;

    if rows_affected == 0 {
        return Err(reject::custom(AppError::NotFound(
            "Public share not found".to_string(),
        )));
    }

    tracing::info!("Successfully deleted public share {}", token_id);

    Ok(reply::with_status(
        reply::json(&serde_json::json!({
            "message": "Public share deleted successfully"
        })),
        warp::http::StatusCode::OK,
    ))
}

/// Get humidor data via public share token (NO AUTHENTICATION REQUIRED)
/// GET /api/v1/shared/humidors/:token
pub async fn get_public_humidor(token_id: Uuid, pool: Pool) -> Result<impl Reply, Rejection> {
    tracing::info!("Public access request for token {}", token_id);

    let client = pool.get().await.map_err(|e| {
        tracing::error!("Failed to get database connection: {}", e);
        reject::custom(AppError::DatabaseError(
            "Failed to connect to database".to_string(),
        ))
    })?;

    // Verify token exists and is not expired, and get include flags
    let token_row = client
        .query_opt(
            "SELECT humidor_id, include_favorites, include_wish_list 
             FROM public_humidor_shares 
             WHERE id = $1 AND (expires_at IS NULL OR expires_at > NOW())",
            &[&token_id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to check share token: {}", e);
            reject::custom(AppError::DatabaseError(
                "Failed to verify token".to_string(),
            ))
        })?;

    let (humidor_id, include_favorites, include_wish_list): (Uuid, bool, bool) = match token_row {
        Some(row) => (row.get(0), row.get(1), row.get(2)),
        None => {
            // Cleanup expired token if it exists
            let _ = client
                .execute(
                    "DELETE FROM public_humidor_shares 
                     WHERE id = $1 AND expires_at <= NOW()",
                    &[&token_id],
                )
                .await;

            tracing::warn!("Invalid or expired token access attempt: {}", token_id);
            return Err(reject::custom(AppError::NotFound(
                "Share link not found or has expired".to_string(),
            )));
        }
    };

    // Fetch humidor details WITHOUT user_id filter (bypass ownership check)
    let humidor_row = client
        .query_opt(
            "SELECT h.id, h.name, h.description, h.image_url, h.created_at,
                    u.username, u.email, u.full_name, u.id as user_id
             FROM humidors h
             INNER JOIN users u ON h.user_id = u.id
             WHERE h.id = $1",
            &[&humidor_id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch humidor: {}", e);
            reject::custom(AppError::DatabaseError(
                "Failed to fetch humidor".to_string(),
            ))
        })?;

    let humidor_row = match humidor_row {
        Some(row) => row,
        None => {
            tracing::error!("Humidor {} not found for valid token", humidor_id);
            return Err(reject::custom(AppError::NotFound(
                "Humidor not found".to_string(),
            )));
        }
    };

    // Fetch cigars in humidor WITHOUT user_id filter
    // Join with organizer tables to get display names
    let cigar_rows = client
        .query(
            "SELECT c.id, c.name, b.name as brand, o.name as origin, c.wrapper, s.name as strength,
                    rg.gauge as ring_gauge, c.length as length_inches, c.quantity, c.notes, c.retail_link, c.image_url
             FROM cigars c
             LEFT JOIN brands b ON c.brand_id = b.id
             LEFT JOIN origins o ON c.origin_id = o.id
             LEFT JOIN strengths s ON c.strength_id = s.id
             LEFT JOIN ring_gauges rg ON c.ring_gauge_id = rg.id
             WHERE c.humidor_id = $1 AND c.is_active = true
             ORDER BY c.name",
            &[&humidor_id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch cigars: {}", e);
            reject::custom(AppError::DatabaseError(
                "Failed to fetch cigars".to_string(),
            ))
        })?;

    // Build response
    let cigars: Vec<PublicCigarResponse> = cigar_rows
        .iter()
        .map(|row| PublicCigarResponse {
            id: row.get(0),
            name: row.get(1),
            brand: row.get(2),
            origin: row.get(3),
            wrapper: row.get(4),
            strength: row.get(5),
            ring_gauge: row.get(6),
            length_inches: row.get(7),
            quantity: row.get(8),
            notes: row.get(9),
            retail_link: row.get(10),
            image_url: row.get(11),
        })
        .collect();

    // Get user_id for favorites/wishlist queries
    let user_id: Uuid = humidor_row.get(8);

    // Fetch favorites if requested
    let favorites = if include_favorites {
        let fav_rows = client
            .query(
                "SELECT c.id, c.name, b.name as brand, o.name as origin, c.wrapper, s.name as strength,
                        rg.gauge as ring_gauge, c.length as length_inches, c.quantity, c.notes, c.retail_link, c.image_url
                 FROM favorites f
                 INNER JOIN cigars c ON f.cigar_id = c.id
                 LEFT JOIN brands b ON c.brand_id = b.id
                 LEFT JOIN origins o ON c.origin_id = o.id
                 LEFT JOIN strengths s ON c.strength_id = s.id
                 LEFT JOIN ring_gauges rg ON c.ring_gauge_id = rg.id
                 WHERE f.user_id = $1 AND c.is_active = true
                 ORDER BY c.name",
                &[&user_id],
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch favorites: {}", e);
                reject::custom(AppError::DatabaseError("Failed to fetch favorites".to_string()))
            })?;

        Some(
            fav_rows
                .iter()
                .map(|row| PublicCigarResponse {
                    id: row.get(0),
                    name: row.get(1),
                    brand: row.get(2),
                    origin: row.get(3),
                    wrapper: row.get(4),
                    strength: row.get(5),
                    ring_gauge: row.get(6),
                    length_inches: row.get(7),
                    quantity: row.get(8),
                    notes: row.get(9),
                    retail_link: row.get(10),
                    image_url: row.get(11),
                })
                .collect::<Vec<_>>(),
        )
    } else {
        None
    };

    // Fetch wish list if requested
    let wish_list = if include_wish_list {
        let wish_rows = client
            .query(
                "SELECT c.id, c.name, b.name as brand, o.name as origin, c.wrapper, s.name as strength,
                        rg.gauge as ring_gauge, c.length as length_inches, 0 as quantity, c.notes, c.retail_link, c.image_url
                 FROM wish_list w
                 INNER JOIN cigars c ON w.cigar_id = c.id
                 LEFT JOIN brands b ON c.brand_id = b.id
                 LEFT JOIN origins o ON c.origin_id = o.id
                 LEFT JOIN strengths s ON c.strength_id = s.id
                 LEFT JOIN ring_gauges rg ON c.ring_gauge_id = rg.id
                 WHERE w.user_id = $1 AND c.is_active = true
                 ORDER BY c.name",
                &[&user_id],
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch wish list: {}", e);
                reject::custom(AppError::DatabaseError("Failed to fetch wish list".to_string()))
            })?;

        Some(
            wish_rows
                .iter()
                .map(|row| PublicCigarResponse {
                    id: row.get(0),
                    name: row.get(1),
                    brand: row.get(2),
                    origin: row.get(3),
                    wrapper: row.get(4),
                    strength: row.get(5),
                    ring_gauge: row.get(6),
                    length_inches: row.get(7),
                    quantity: row.get(8), // 0 for wishlist items
                    notes: row.get(9),
                    retail_link: row.get(10),
                    image_url: row.get(11),
                })
                .collect::<Vec<_>>(),
        )
    } else {
        None
    };

    let humidor = PublicHumidorResponse {
        id: humidor_row.get(0),
        name: humidor_row.get(1),
        description: humidor_row.get(2),
        image_url: humidor_row.get(3),
        created_at: humidor_row.get(4),
        owner: PublicUserInfo {
            username: humidor_row.get(5),
            full_name: humidor_row.get(7),
        },
        cigar_count: cigars.len(),
        cigars,
        favorites,
        wish_list,
    };

    tracing::info!(
        "Successfully served public humidor {} via token {}",
        humidor_id,
        token_id
    );

    Ok(reply::json(&humidor))
}
