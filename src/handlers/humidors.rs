use crate::handlers::humidor_shares::can_view_humidor;
use crate::middleware::AuthContext;
use crate::models::{CreateHumidorRequest, Humidor, UpdateHumidorRequest};
use crate::validation::Validate;
use crate::DbPool;
use serde_json::json;
use std::convert::Infallible;
use uuid::Uuid;
use warp::{http::StatusCode, reply, Reply};

pub async fn get_humidors(auth: AuthContext, pool: DbPool) -> Result<impl Reply, Infallible> {
    let db = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!(error = %e, "Failed to get database connection");
            return Ok(reply::with_status(
                reply::json(&json!({"error": "Database connection failed"})),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    let user_id = auth.user_id;

    // Get humidors owned by user UNION with humidors shared with user
    let query = "
        SELECT DISTINCT h.id, h.user_id, h.name, h.description, h.capacity, h.target_humidity, h.location, h.created_at, h.updated_at,
               CASE WHEN h.user_id = $1 THEN true ELSE false END as is_owner,
               COALESCE(hs.permission_level, 'full') as permission_level
        FROM humidors h
        LEFT JOIN humidor_shares hs ON h.id = hs.humidor_id AND hs.shared_with_user_id = $1
        WHERE h.user_id = $1 OR hs.shared_with_user_id = $1
        ORDER BY h.created_at ASC
    ";

    match db.query(query, &[&user_id]).await {
        Ok(rows) => {
            let humidors: Vec<Humidor> = rows
                .iter()
                .map(|row| Humidor {
                    id: row.get(0),
                    user_id: row.get(1),
                    name: row.get(2),
                    description: row.get(3),
                    capacity: row.get(4),
                    target_humidity: row.get(5),
                    location: row.get(6),
                    created_at: row.get(7),
                    updated_at: row.get(8),
                    is_owner: Some(row.get(9)),
                    permission_level: Some(row.get(10)),
                })
                .collect();

            Ok(reply::with_status(reply::json(&humidors), StatusCode::OK))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            let error_response = json!({
                "error": "Failed to fetch humidors",
                "details": e.to_string()
            });
            Ok(reply::with_status(
                reply::json(&error_response),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

pub async fn get_humidor(
    id: Uuid,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Infallible> {
    let db = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!(error = %e, "Failed to get database connection");
            return Ok(reply::with_status(
                reply::json(&json!({"error": "Database connection failed"})),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    let user_id = auth.user_id;

    // Check if user can view this humidor (owner or has share access)
    match can_view_humidor(&pool, &user_id, &id).await {
        Ok(true) => {
            // User has access, fetch the humidor
            let query = "
                SELECT id, user_id, name, description, capacity, target_humidity, location, created_at, updated_at
                FROM humidors 
                WHERE id = $1
            ";

            match db.query_opt(query, &[&id]).await {
                Ok(Some(row)) => {
                    let humidor = Humidor {
                        id: row.get(0),
                        user_id: row.get(1),
                        name: row.get(2),
                        description: row.get(3),
                        capacity: row.get(4),
                        target_humidity: row.get(5),
                        location: row.get(6),
                        created_at: row.get(7),
                        updated_at: row.get(8),
                        is_owner: None,
                        permission_level: None,
                    };

                    Ok(reply::with_status(reply::json(&humidor), StatusCode::OK))
                }
                Ok(None) => {
                    let error_response = json!({
                        "error": "Humidor not found"
                    });
                    Ok(reply::with_status(
                        reply::json(&error_response),
                        StatusCode::NOT_FOUND,
                    ))
                }
                Err(e) => {
                    tracing::error!(error = %e, "Database error");
                    let error_response = json!({
                        "error": "Failed to fetch humidor",
                        "details": e.to_string()
                    });
                    Ok(reply::with_status(
                        reply::json(&error_response),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        }
        Ok(false) => {
            // User does not have access
            let error_response = json!({
                "error": "Forbidden: You do not have access to this humidor"
            });
            Ok(reply::with_status(
                reply::json(&error_response),
                StatusCode::FORBIDDEN,
            ))
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to check humidor access");
            let error_response = json!({
                "error": "Failed to check access"
            });
            Ok(reply::with_status(
                reply::json(&error_response),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

pub async fn create_humidor(
    request: CreateHumidorRequest,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Infallible> {
    let db = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!(error = %e, "Failed to get database connection");
            return Ok(reply::with_status(
                reply::json(&json!({"error": "Database connection failed"})),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Validate input
    if let Err(e) = request.validate() {
        return Ok(reply::with_status(
            reply::json(&json!({"error": e.to_string()})),
            StatusCode::BAD_REQUEST,
        ));
    }

    let user_id = auth.user_id;
    let humidor_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let query = "
        INSERT INTO humidors (id, user_id, name, description, capacity, target_humidity, location, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, user_id, name, description, capacity, target_humidity, location, created_at, updated_at
    ";

    match db
        .query_one(
            query,
            &[
                &humidor_id,
                &user_id,
                &request.name,
                &request.description,
                &request.capacity,
                &request.target_humidity,
                &request.location,
                &now,
                &now,
            ],
        )
        .await
    {
        Ok(row) => {
            let humidor = Humidor {
                id: row.get(0),
                user_id: row.get(1),
                name: row.get(2),
                description: row.get(3),
                capacity: row.get(4),
                target_humidity: row.get(5),
                location: row.get(6),
                created_at: row.get(7),
                updated_at: row.get(8),
                is_owner: Some(true), // User creating is always owner
                permission_level: Some("full".to_string()),
            };

            Ok(reply::with_status(
                reply::json(&humidor),
                StatusCode::CREATED,
            ))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            let error_response = json!({
                "error": "Failed to create humidor",
                "details": e.to_string()
            });
            Ok(reply::with_status(
                reply::json(&error_response),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

pub async fn update_humidor(
    id: Uuid,
    request: UpdateHumidorRequest,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Infallible> {
    let db = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!(error = %e, "Failed to get database connection");
            return Ok(reply::with_status(
                reply::json(&json!({"error": "Database connection failed"})),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Validate input
    if let Err(e) = request.validate() {
        return Ok(reply::with_status(
            reply::json(&json!({"error": e.to_string()})),
            StatusCode::BAD_REQUEST,
        ));
    }

    let user_id = auth.user_id;
    let now = chrono::Utc::now();

    let query = "
        UPDATE humidors 
        SET name = $3, description = $4, capacity = $5, target_humidity = $6, location = $7, updated_at = $8
        WHERE id = $1 AND user_id = $2
        RETURNING id, user_id, name, description, capacity, target_humidity, location, created_at, updated_at
    ";

    match db
        .query_opt(
            query,
            &[
                &id,
                &user_id,
                &request.name,
                &request.description,
                &request.capacity,
                &request.target_humidity,
                &request.location,
                &now,
            ],
        )
        .await
    {
        Ok(Some(row)) => {
            let humidor = Humidor {
                id: row.get(0),
                user_id: row.get(1),
                name: row.get(2),
                description: row.get(3),
                capacity: row.get(4),
                target_humidity: row.get(5),
                location: row.get(6),
                created_at: row.get(7),
                updated_at: row.get(8),
                is_owner: None, // Not needed for update response
                permission_level: None,
            };

            Ok(reply::with_status(reply::json(&humidor), StatusCode::OK))
        }
        Ok(None) => {
            let error_response = json!({
                "error": "Humidor not found"
            });
            Ok(reply::with_status(
                reply::json(&error_response),
                StatusCode::NOT_FOUND,
            ))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            let error_response = json!({
                "error": "Failed to update humidor",
                "details": e.to_string()
            });
            Ok(reply::with_status(
                reply::json(&error_response),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

pub async fn delete_humidor(
    id: Uuid,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Infallible> {
    let db = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!(error = %e, "Failed to get database connection");
            return Ok(reply::with_status(
                reply::json(&json!({"error": "Database connection failed"})),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    let user_id = auth.user_id;
    let query = "DELETE FROM humidors WHERE id = $1 AND user_id = $2";

    match db.execute(query, &[&id, &user_id]).await {
        Ok(rows_affected) => {
            if rows_affected == 0 {
                let error_response = json!({
                    "error": "Humidor not found"
                });
                Ok(reply::with_status(
                    reply::json(&error_response),
                    StatusCode::NOT_FOUND,
                ))
            } else {
                let success_response = json!({
                    "message": "Humidor deleted successfully"
                });
                Ok(reply::with_status(
                    reply::json(&success_response),
                    StatusCode::OK,
                ))
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            let error_response = json!({
                "error": "Failed to delete humidor",
                "details": e.to_string()
            });
            Ok(reply::with_status(
                reply::json(&error_response),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

pub async fn get_humidor_cigars(
    humidor_id: Uuid,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Infallible> {
    let db = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!(error = %e, "Failed to get database connection");
            return Ok(reply::with_status(
                reply::json(&json!({"error": "Database connection failed"})),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    let user_id = auth.user_id;
    // First verify the humidor belongs to the user
    let humidor_check = "SELECT id FROM humidors WHERE id = $1 AND user_id = $2";
    match db.query_opt(humidor_check, &[&humidor_id, &user_id]).await {
        Ok(Some(_)) => {
            // Humidor exists and belongs to user, get cigars
            let query = "
                SELECT c.id, c.humidor_id, c.brand, c.name, c.size, c.wrapper, 
                       c.strength, c.origin, c.price, c.purchase_date, c.notes, c.quantity, 
                       c.ring_gauge, c.length, c.created_at, c.updated_at
                FROM cigars c 
                WHERE c.humidor_id = $1
                ORDER BY c.created_at DESC
            ";

            match db.query(query, &[&humidor_id]).await {
                Ok(rows) => {
                    let cigars: Vec<serde_json::Value> = rows
                        .iter()
                        .map(|row| json!({
                            "id": row.get::<_, Uuid>(0),
                            "humidor_id": row.get::<_, Option<Uuid>>(1),
                            "brand": row.get::<_, String>(2),
                            "name": row.get::<_, String>(3),
                            "size": row.get::<_, String>(4),
                            "wrapper": row.get::<_, Option<String>>(5),
                            "strength": row.get::<_, String>(6),
                            "origin": row.get::<_, String>(7),
                            "price": row.get::<_, Option<f64>>(8),
                            "purchase_date": row.get::<_, Option<chrono::DateTime<chrono::Utc>>>(9),
                            "notes": row.get::<_, Option<String>>(10),
                            "quantity": row.get::<_, i32>(11),
                            "ring_gauge": row.get::<_, Option<i32>>(12),
                            "length": row.get::<_, Option<f64>>(13),
                            "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>(14),
                            "updated_at": row.get::<_, chrono::DateTime<chrono::Utc>>(15),
                        }))
                        .collect();

                    Ok(reply::with_status(reply::json(&cigars), StatusCode::OK))
                }
                Err(e) => {
                    tracing::error!(error = %e, "Database error");
                    let error_response = json!({
                        "error": "Failed to fetch cigars",
                        "details": e.to_string()
                    });
                    Ok(reply::with_status(
                        reply::json(&error_response),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        }
        Ok(None) => {
            let error_response = json!({
                "error": "Humidor not found"
            });
            Ok(reply::with_status(
                reply::json(&error_response),
                StatusCode::NOT_FOUND,
            ))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            let error_response = json!({
                "error": "Failed to verify humidor access",
                "details": e.to_string()
            });
            Ok(reply::with_status(
                reply::json(&error_response),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}
