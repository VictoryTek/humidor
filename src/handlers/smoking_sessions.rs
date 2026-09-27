use crate::errors::AppError;
use crate::middleware::auth::AuthContext;
use crate::models::{
    CreateSmokingSession, SmokingSession, SmokingSessionResponse, SmokingSessionWithCigar,
};
use crate::validation::Validate;
use crate::{DbPool, handlers::cigars::verify_cigar_ownership};
use uuid::Uuid;
use warp::{Rejection, Reply};

fn row_to_session(row: &tokio_postgres::Row) -> SmokingSession {
    SmokingSession {
        id: row.get(0),
        user_id: row.get(1),
        cigar_id: row.get(2),
        smoked_at: row.get(3),
        rating: row.get(4),
        duration_minutes: row.get(5),
        pairing: row.get(6),
        notes: row.get(7),
        created_at: row.get(8),
    }
}

const SESSION_COLUMNS: &str =
    "id, user_id, cigar_id, smoked_at, rating, duration_minutes, pairing, notes, created_at";

/// Log a smoking session for a cigar. Requires edit permission (same rule as other cigar
/// mutations, via `verify_cigar_ownership`). In one transaction: insert the session, then
/// decrement the cigar's quantity by exactly 1 using the same is_active flip-at-zero logic as
/// `update_cigar`. Decrementing below 0 is rejected (400), not clamped: a "Smoke One" action on an
/// already-out-of-stock cigar is a usage error, not something to silently absorb.
/// POST /api/v1/cigars/:id/sessions
pub async fn create_session(
    cigar_id: Uuid,
    auth: AuthContext,
    request: CreateSmokingSession,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    request.validate().map_err(warp::reject::custom)?;

    verify_cigar_ownership(&pool, cigar_id, auth.user_id, true)
        .await
        .map_err(warp::reject::custom)?;

    let mut db = pool.get().await.map_err(|e| {
        warp::reject::custom(AppError::DatabaseError(format!(
            "Failed to get database connection: {e}"
        )))
    })?;

    let tx = db.transaction().await.map_err(|e| {
        warp::reject::custom(AppError::DatabaseError(format!(
            "Failed to start transaction: {e}"
        )))
    })?;

    let quantity_row = tx
        .query_opt(
            "SELECT quantity FROM cigars WHERE id = $1 FOR UPDATE",
            &[&cigar_id],
        )
        .await
        .map_err(|e| warp::reject::custom(AppError::DatabaseError(e.to_string())))?
        .ok_or_else(|| warp::reject::custom(AppError::NotFound("Cigar".to_string())))?;
    let current_quantity: i32 = quantity_row.get(0);

    if current_quantity <= 0 {
        return Err(warp::reject::custom(AppError::BadRequest(
            "This cigar has no quantity remaining to smoke".to_string(),
        )));
    }

    let session_id = Uuid::new_v4();
    let session_row = tx
        .query_one(
            &format!(
                "INSERT INTO smoking_sessions
                    (id, user_id, cigar_id, smoked_at, rating, duration_minutes, pairing, notes)
                 VALUES ($1, $2, $3, COALESCE($4, NOW()), $5, $6, $7, $8)
                 RETURNING {SESSION_COLUMNS}"
            ),
            &[
                &session_id,
                &auth.user_id,
                &cigar_id,
                &request.smoked_at,
                &request.rating,
                &request.duration_minutes,
                &request.pairing,
                &request.notes,
            ],
        )
        .await
        .map_err(|e| warp::reject::custom(AppError::DatabaseError(e.to_string())))?;

    let cigar_row = tx
        .query_one(
            "UPDATE cigars SET
                quantity = quantity - 1,
                is_active = CASE WHEN quantity - 1 = 0 THEN false ELSE is_active END,
                updated_at = NOW()
             WHERE id = $1
             RETURNING quantity, is_active",
            &[&cigar_id],
        )
        .await
        .map_err(|e| warp::reject::custom(AppError::DatabaseError(e.to_string())))?;

    tx.commit().await.map_err(|e| {
        warp::reject::custom(AppError::DatabaseError(format!(
            "Failed to commit transaction: {e}"
        )))
    })?;

    Ok(warp::reply::with_status(
        warp::reply::json(&SmokingSessionResponse {
            session: row_to_session(&session_row),
            cigar_quantity: cigar_row.get(0),
            cigar_is_active: cigar_row.get(1),
        }),
        warp::http::StatusCode::CREATED,
    ))
}

/// List a cigar's smoking sessions, newest first. Requires view permission.
/// GET /api/v1/cigars/:id/sessions
pub async fn get_cigar_sessions(
    cigar_id: Uuid,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    verify_cigar_ownership(&pool, cigar_id, auth.user_id, false)
        .await
        .map_err(warp::reject::custom)?;

    let db = pool.get().await.map_err(|e| {
        warp::reject::custom(AppError::DatabaseError(format!(
            "Failed to get database connection: {e}"
        )))
    })?;

    let rows = db
        .query(
            &format!(
                "SELECT {SESSION_COLUMNS} FROM smoking_sessions WHERE cigar_id = $1 ORDER BY smoked_at DESC"
            ),
            &[&cigar_id],
        )
        .await
        .map_err(|e| warp::reject::custom(AppError::DatabaseError(e.to_string())))?;

    let sessions: Vec<SmokingSession> = rows.iter().map(row_to_session).collect();
    Ok(warp::reply::json(&sessions))
}

#[derive(serde::Serialize)]
pub struct SessionHistoryResponse {
    pub sessions: Vec<SmokingSessionWithCigar>,
    pub total: i64,
}

/// The requesting user's own logged sessions across all cigars, newest first, paginated the same
/// way `get_cigars` is (`limit`/`offset`, default page size 20).
/// GET /api/v1/sessions?limit=&offset=
pub async fn get_my_sessions(
    params: std::collections::HashMap<String, String>,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    let limit: i64 = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .filter(|v| *v > 0 && *v <= 200)
        .unwrap_or(20);
    let offset: i64 = params
        .get("offset")
        .and_then(|v| v.parse().ok())
        .filter(|v| *v >= 0)
        .unwrap_or(0);

    let db = pool.get().await.map_err(|e| {
        warp::reject::custom(AppError::DatabaseError(format!(
            "Failed to get database connection: {e}"
        )))
    })?;

    let total: i64 = db
        .query_one(
            "SELECT COUNT(*) FROM smoking_sessions WHERE user_id = $1",
            &[&auth.user_id],
        )
        .await
        .map_err(|e| warp::reject::custom(AppError::DatabaseError(e.to_string())))?
        .get(0);

    let rows = db
        .query(
            "SELECT s.id, s.user_id, s.cigar_id, s.smoked_at, s.rating, s.duration_minutes,
                    s.pairing, s.notes, s.created_at, c.name, b.name
             FROM smoking_sessions s
             JOIN cigars c ON c.id = s.cigar_id
             LEFT JOIN brands b ON b.id = c.brand_id
             WHERE s.user_id = $1
             ORDER BY s.smoked_at DESC
             LIMIT $2 OFFSET $3",
            &[&auth.user_id, &limit, &offset],
        )
        .await
        .map_err(|e| warp::reject::custom(AppError::DatabaseError(e.to_string())))?;

    let sessions: Vec<SmokingSessionWithCigar> = rows
        .iter()
        .map(|row| SmokingSessionWithCigar {
            session: row_to_session(row),
            cigar_name: row.get(9),
            brand_name: row.get(10),
        })
        .collect();

    Ok(warp::reply::json(&SessionHistoryResponse {
        sessions,
        total,
    }))
}
