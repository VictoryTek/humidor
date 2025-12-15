use chrono::Utc;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;
use warp::{Rejection, Reply};

use crate::{
    DbPool,
    errors::AppError,
    handlers::humidor_shares::{can_edit_humidor, can_view_humidor},
    middleware::auth::AuthContext,
    models::*,
    validation::Validate,
};

#[derive(Debug, Serialize)]
pub struct CigarResponse {
    pub cigars: Vec<CigarWithNames>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

/// Helper function to verify that a humidor belongs to the authenticated user OR is shared with them with edit permissions
async fn verify_humidor_ownership(
    pool: &DbPool,
    humidor_id: Option<Uuid>,
    user_id: Uuid,
    require_edit: bool,
) -> Result<(), AppError> {
    if let Some(hid) = humidor_id {
        // First check if user owns the humidor
        let db = pool.get().await.map_err(|e| {
            tracing::error!(error = %e, "Failed to get database connection");
            AppError::DatabaseError("Failed to connect to database".to_string())
        })?;

        let check_query = "SELECT EXISTS(SELECT 1 FROM humidors WHERE id = $1 AND user_id = $2)";
        match db.query_one(check_query, &[&hid, &user_id]).await {
            Ok(row) => {
                let is_owner: bool = row.get(0);
                if is_owner {
                    return Ok(()); // Owner has full access
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to verify humidor ownership");
                return Err(AppError::DatabaseError(
                    "Failed to verify humidor access".to_string(),
                ));
            }
        }

        // Not owner, check if it's shared with appropriate permissions
        if require_edit {
            if can_edit_humidor(pool, &user_id, &hid).await? {
                return Ok(());
            }
        } else if can_view_humidor(pool, &user_id, &hid).await? {
            return Ok(());
        }

        Err(AppError::Forbidden(
            "You do not have access to this humidor".to_string(),
        ))
    } else {
        // No humidor specified is okay for some operations (e.g., listing all cigars across humidors)
        Ok(())
    }
}

/// Helper function to verify that a cigar belongs to the authenticated user (through its humidor) OR is shared with them
async fn verify_cigar_ownership(
    pool: &DbPool,
    cigar_id: Uuid,
    user_id: Uuid,
    require_edit: bool,
) -> Result<(), AppError> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        AppError::DatabaseError("Failed to connect to database".to_string())
    })?;

    // Check if user owns the cigar through their humidor
    let check_query = "
        SELECT EXISTS(
            SELECT 1 FROM cigars c
            INNER JOIN humidors h ON c.humidor_id = h.id
            WHERE c.id = $1 AND h.user_id = $2
        )
    ";
    match db.query_one(check_query, &[&cigar_id, &user_id]).await {
        Ok(row) => {
            let is_owner: bool = row.get(0);
            if is_owner {
                return Ok(()); // Owner has full access
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to verify cigar ownership");
            return Err(AppError::DatabaseError(
                "Failed to verify cigar access".to_string(),
            ));
        }
    }

    // Check if cigar is in user's wish list (wish list cigars have NULL humidor_id)
    let wish_list_query = "
        SELECT EXISTS(
            SELECT 1 FROM wish_list
            WHERE cigar_id = $1 AND user_id = $2
        )
    ";
    match db.query_one(wish_list_query, &[&cigar_id, &user_id]).await {
        Ok(row) => {
            let in_wish_list: bool = row.get(0);
            if in_wish_list {
                return Ok(()); // User can access their wish list cigars
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to check wish list");
            return Err(AppError::DatabaseError(
                "Failed to verify cigar access".to_string(),
            ));
        }
    }

    // Not owner or in wish list, get the humidor_id and check if it's shared
    let humidor_query = "SELECT humidor_id FROM cigars WHERE id = $1";
    match db.query_opt(humidor_query, &[&cigar_id]).await {
        Ok(Some(row)) => {
            let humidor_id: Option<Uuid> = row.get(0);

            // If humidor_id is NULL, cigar doesn't belong to any humidor
            if let Some(hum_id) = humidor_id {
                // Check if humidor is shared with appropriate permissions
                if require_edit {
                    if can_edit_humidor(pool, &user_id, &hum_id).await? {
                        return Ok(());
                    }
                } else if can_view_humidor(pool, &user_id, &hum_id).await? {
                    return Ok(());
                }
            }
        }
        Ok(None) => {
            return Err(AppError::NotFound("Cigar not found".to_string()));
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to get cigar's humidor");
            return Err(AppError::DatabaseError(
                "Failed to verify cigar access".to_string(),
            ));
        }
    }

    Err(AppError::Forbidden(
        "You do not have access to this cigar".to_string(),
    ))
}

pub async fn get_cigars(
    params: std::collections::HashMap<String, String>,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    use crate::errors::AppError;
    use tokio_postgres::types::ToSql;

    let start_time = std::time::Instant::now();

    // Acquire a connection from the pool
    let db = pool.get().await.map_err(|e| {
        warp::reject::custom(AppError::DatabaseError(format!(
            "Connection pool error: {}",
            e
        )))
    })?;

    // Parse pagination parameters
    let page = params
        .get("page")
        .and_then(|p| p.parse::<i64>().ok())
        .unwrap_or(1)
        .max(1); // Ensure page is at least 1

    let page_size = params
        .get("page_size")
        .and_then(|ps| ps.parse::<i64>().ok())
        .unwrap_or(50)
        .clamp(1, 100); // Allow 1-100 items per page

    let offset = (page - 1) * page_size;

    // Build query with JOINs to include organizer names from the humidor owner's organizers
    // This ensures shared humidors display correctly even when the viewer doesn't have those organizers
    let base_query = "
        SELECT 
            c.id, c.humidor_id, c.brand_id, c.name, c.size_id, c.strength_id, c.origin_id, 
            c.wrapper, c.binder, c.filler, c.price, c.purchase_date, c.notes, c.quantity, 
            c.ring_gauge_id, c.length, c.image_url, c.retail_link, c.is_active, c.created_at, c.updated_at,
            b.name as brand_name, s.name as size_name, st.name as strength_name, 
            o.name as origin_name, rg.gauge as ring_gauge
        FROM cigars c 
        INNER JOIN humidors h ON c.humidor_id = h.id 
        LEFT JOIN humidor_shares hs ON c.humidor_id = hs.humidor_id AND hs.shared_with_user_id = $1
        LEFT JOIN brands b ON c.brand_id = b.id AND b.user_id = h.user_id
        LEFT JOIN sizes s ON c.size_id = s.id AND s.user_id = h.user_id
        LEFT JOIN strengths st ON c.strength_id = st.id AND st.user_id = h.user_id
        LEFT JOIN origins o ON c.origin_id = o.id AND o.user_id = h.user_id
        LEFT JOIN ring_gauges rg ON c.ring_gauge_id = rg.id AND rg.user_id = h.user_id
    ";
    let count_query = "SELECT COUNT(*) FROM cigars c INNER JOIN humidors h ON c.humidor_id = h.id LEFT JOIN humidor_shares hs ON c.humidor_id = hs.humidor_id AND hs.shared_with_user_id = $1";
    let mut conditions = Vec::new();
    let mut param_values: Vec<Box<dyn ToSql + Sync + Send>> = Vec::new();
    let mut param_counter = 1;

    // CRITICAL: Filter by user-owned humidors OR humidors shared with user
    // $1 is already used in the LEFT JOIN above
    conditions.push(format!(
        "(h.user_id = ${} OR hs.shared_with_user_id = ${})",
        param_counter, param_counter
    ));
    param_values.push(Box::new(auth.user_id));
    param_counter += 1;

    // Check for humidor_id filter
    if let Some(humidor_id_str) = params.get("humidor_id")
        && let Ok(humidor_uuid) = Uuid::parse_str(humidor_id_str)
    {
        // Verify the humidor belongs to the user or is shared (view permission is enough)
        if let Err(e) =
            verify_humidor_ownership(&pool, Some(humidor_uuid), auth.user_id, false).await
        {
            return Err(warp::reject::custom(e));
        }
        conditions.push(format!("c.humidor_id = ${}", param_counter));
        param_values.push(Box::new(humidor_uuid));
        param_counter += 1;
    }

    // Check for organizer filters (brand, size, origin, strength, ring_gauge)
    if let Some(brand_id_str) = params.get("brand_id")
        && let Ok(brand_uuid) = Uuid::parse_str(brand_id_str)
    {
        conditions.push(format!("c.brand_id = ${}", param_counter));
        param_values.push(Box::new(brand_uuid));
        param_counter += 1;
    }
    if let Some(size_id_str) = params.get("size_id")
        && let Ok(size_uuid) = Uuid::parse_str(size_id_str)
    {
        conditions.push(format!("c.size_id = ${}", param_counter));
        param_values.push(Box::new(size_uuid));
        param_counter += 1;
    }
    if let Some(origin_id_str) = params.get("origin_id")
        && let Ok(origin_uuid) = Uuid::parse_str(origin_id_str)
    {
        conditions.push(format!("c.origin_id = ${}", param_counter));
        param_values.push(Box::new(origin_uuid));
        param_counter += 1;
    }
    if let Some(strength_id_str) = params.get("strength_id")
        && let Ok(strength_uuid) = Uuid::parse_str(strength_id_str)
    {
        conditions.push(format!("c.strength_id = ${}", param_counter));
        param_values.push(Box::new(strength_uuid));
        param_counter += 1;
    }
    if let Some(ring_gauge_id_str) = params.get("ring_gauge_id")
        && let Ok(ring_gauge_uuid) = Uuid::parse_str(ring_gauge_id_str)
    {
        conditions.push(format!("c.ring_gauge_id = ${}", param_counter));
        param_values.push(Box::new(ring_gauge_uuid));
        param_counter += 1;
    }

    // Build WHERE clause
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    // Get total count with same filters
    let total_count_query = format!("{}{}", count_query, where_clause);

    // Convert boxed parameters to references for query execution
    let param_refs: Vec<&(dyn ToSql + Sync)> = param_values
        .iter()
        .map(|b| &**b as &(dyn ToSql + Sync))
        .collect();

    // Execute count query
    let total = match db.query_one(&total_count_query, &param_refs[..]).await {
        Ok(row) => row.get::<_, i64>(0),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get total count");
            return Err(warp::reject::custom(AppError::DatabaseError(
                "Failed to count cigars".to_string(),
            )));
        }
    };

    let total_pages = (total as f64 / page_size as f64).ceil() as i64;

    // Add pagination parameters to the param list
    param_values.push(Box::new(page_size));
    param_values.push(Box::new(offset));

    // Rebuild param_refs with new values
    let param_refs: Vec<&(dyn ToSql + Sync)> = param_values
        .iter()
        .map(|b| &**b as &(dyn ToSql + Sync))
        .collect();

    // Build the final query with pagination
    let query = format!(
        "{}{} ORDER BY is_active DESC, created_at DESC LIMIT ${} OFFSET ${}",
        base_query,
        where_clause,
        param_counter,
        param_counter + 1
    );

    match db.query(&query, &param_refs[..]).await {
        Ok(rows) => {
            let mut cigars = Vec::new();
            for row in rows {
                let cigar = Cigar {
                    id: row.get(0),
                    humidor_id: row.get(1),
                    brand_id: row.get(2),
                    name: row.get(3),
                    size_id: row.get(4),
                    strength_id: row.get(5),
                    origin_id: row.get(6),
                    wrapper: row.get(7),
                    binder: row.get(8),
                    filler: row.get(9),
                    price: row.get(10),
                    purchase_date: row.get(11),
                    notes: row.get(12),
                    quantity: row.get(13),
                    ring_gauge_id: row.get(14),
                    length: row.get(15),
                    image_url: row.get(16),
                    retail_link: row.get(17),
                    is_active: row.get(18),
                    created_at: row.get(19),
                    updated_at: row.get(20),
                };

                let cigar_with_names = CigarWithNames {
                    cigar,
                    brand_name: row.get(21),
                    size_name: row.get(22),
                    strength_name: row.get(23),
                    origin_name: row.get(24),
                    ring_gauge: row.get(25),
                };

                cigars.push(cigar_with_names);
            }

            let elapsed = start_time.elapsed();
            if elapsed.as_millis() > 100 {
                tracing::warn!(
                    duration_ms = elapsed.as_millis(),
                    total_results = total,
                    page = page,
                    "Slow query detected in get_cigars"
                );
            } else {
                tracing::debug!(
                    duration_ms = elapsed.as_millis(),
                    total_results = total,
                    page = page,
                    "Query completed"
                );
            }

            let response = CigarResponse {
                cigars,
                total,
                page,
                page_size,
                total_pages,
            };

            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error fetching cigars");
            Ok(warp::reply::json(
                &json!({"error": "Failed to fetch cigars"}),
            ))
        }
    }
}

pub async fn create_cigar(
    create_cigar: CreateCigar,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    // Validate input
    create_cigar.validate().map_err(warp::reject::custom)?;

    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    // CRITICAL: Verify the humidor belongs to the authenticated user or is shared with edit permission
    verify_humidor_ownership(&pool, create_cigar.humidor_id, auth.user_id, true)
        .await
        .map_err(warp::reject::custom)?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    match db.query_one(
        "INSERT INTO cigars (id, humidor_id, brand_id, name, size_id, strength_id, origin_id, wrapper, binder, filler, price, purchase_date, notes, quantity, ring_gauge_id, length, image_url, retail_link, is_active, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, true, $19, $20)
         RETURNING id, humidor_id, brand_id, name, size_id, strength_id, origin_id, wrapper, binder, filler, price, purchase_date, notes, quantity, ring_gauge_id, length, image_url, retail_link, is_active, created_at, updated_at",
        &[&id, &create_cigar.humidor_id, &create_cigar.brand_id, &create_cigar.name, &create_cigar.size_id, &create_cigar.strength_id, &create_cigar.origin_id,
          &create_cigar.wrapper, &create_cigar.binder, &create_cigar.filler, &create_cigar.price, &create_cigar.purchase_date,
          &create_cigar.notes, &create_cigar.quantity, &create_cigar.ring_gauge_id, &create_cigar.length, &create_cigar.image_url, &create_cigar.retail_link, &now, &now]
    ).await {
        Ok(row) => {
            let cigar = Cigar {
                id: row.get(0),
                humidor_id: row.get(1),
                brand_id: row.get(2),
                name: row.get(3),
                size_id: row.get(4),
                strength_id: row.get(5),
                origin_id: row.get(6),
                wrapper: row.get(7),
                binder: row.get(8),
                filler: row.get(9),
                price: row.get(10),
                purchase_date: row.get(11),
                notes: row.get(12),
                quantity: row.get(13),
                ring_gauge_id: row.get(14),
                length: row.get(15),
                image_url: row.get(16),
                retail_link: row.get(17),
                is_active: row.get(18),
                created_at: row.get(19),
                updated_at: row.get(20),
            };
            Ok(warp::reply::json(&cigar))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(&json!({"error": "Failed to create cigar"})))
        }
    }
}

pub async fn get_cigar(id: Uuid, auth: AuthContext, pool: DbPool) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    // CRITICAL: Verify the cigar belongs to the user or is shared (view permission is enough)
    verify_cigar_ownership(&pool, id, auth.user_id, false)
        .await
        .map_err(warp::reject::custom)?;

    match db.query_one(
        "SELECT id, humidor_id, brand_id, name, size_id, strength_id, origin_id, wrapper, binder, filler, price, purchase_date, notes, quantity, ring_gauge_id, length, image_url, retail_link, is_active, created_at, updated_at FROM cigars WHERE id = $1",
        &[&id]
    ).await {
        Ok(row) => {
            let cigar = Cigar {
                id: row.get(0),
                humidor_id: row.get(1),
                brand_id: row.get(2),
                name: row.get(3),
                size_id: row.get(4),
                strength_id: row.get(5),
                origin_id: row.get(6),
                wrapper: row.get(7),
                binder: row.get(8),
                filler: row.get(9),
                price: row.get(10),
                purchase_date: row.get(11),
                notes: row.get(12),
                quantity: row.get(13),
                ring_gauge_id: row.get(14),
                length: row.get(15),
                image_url: row.get(16),
                retail_link: row.get(17),
                is_active: row.get(18),
                created_at: row.get(19),
                updated_at: row.get(20),
            };
            Ok(warp::reply::json(&cigar))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(&json!({"error": "Cigar not found"})))
        }
    }
}

pub async fn update_cigar(
    id: Uuid,
    update_cigar: UpdateCigar,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    // Validate input
    update_cigar.validate().map_err(warp::reject::custom)?;

    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    // CRITICAL: Verify the cigar belongs to the user or is shared with edit permission
    verify_cigar_ownership(&pool, id, auth.user_id, true)
        .await
        .map_err(warp::reject::custom)?;

    // If updating humidor_id, verify the new humidor also belongs to the user or is shared with edit permission
    if let Some(new_humidor_id) = update_cigar.humidor_id {
        verify_humidor_ownership(&pool, Some(new_humidor_id), auth.user_id, true)
            .await
            .map_err(warp::reject::custom)?;
    }

    let now = Utc::now();

    match db.query_one(
        "UPDATE cigars SET
         humidor_id = COALESCE($2, humidor_id),
         brand_id = COALESCE($3, brand_id),
         name = COALESCE($4, name),
         size_id = COALESCE($5, size_id),
         strength_id = COALESCE($6, strength_id),
         origin_id = COALESCE($7, origin_id),
         wrapper = COALESCE($8, wrapper),
         binder = COALESCE($9, binder),
         filler = COALESCE($10, filler),
         price = COALESCE($11, price),
         purchase_date = COALESCE($12, purchase_date),
         notes = COALESCE($13, notes),
         quantity = COALESCE($14, quantity),
         ring_gauge_id = COALESCE($15, ring_gauge_id),
         length = COALESCE($16, length),
         image_url = COALESCE($17, image_url),
         retail_link = COALESCE($18, retail_link),
         is_active = CASE
             WHEN $14 IS NOT NULL AND $14 = 0 THEN false
             WHEN $14 IS NOT NULL AND $14 > 0 THEN true
             ELSE is_active
         END,
         updated_at = $19
         WHERE id = $1
         RETURNING id, humidor_id, brand_id, name, size_id, strength_id, origin_id, wrapper, binder, filler, price, purchase_date, notes, quantity, ring_gauge_id, length, image_url, retail_link, is_active, created_at, updated_at",
        &[&id, &update_cigar.humidor_id, &update_cigar.brand_id, &update_cigar.name, &update_cigar.size_id, &update_cigar.strength_id, &update_cigar.origin_id,
          &update_cigar.wrapper, &update_cigar.binder, &update_cigar.filler, &update_cigar.price, &update_cigar.purchase_date,
          &update_cigar.notes, &update_cigar.quantity, &update_cigar.ring_gauge_id, &update_cigar.length, &update_cigar.image_url, &update_cigar.retail_link, &now]
    ).await {
        Ok(row) => {
            let cigar = Cigar {
                id: row.get(0),
                humidor_id: row.get(1),
                brand_id: row.get(2),
                name: row.get(3),
                size_id: row.get(4),
                strength_id: row.get(5),
                origin_id: row.get(6),
                wrapper: row.get(7),
                binder: row.get(8),
                filler: row.get(9),
                price: row.get(10),
                purchase_date: row.get(11),
                notes: row.get(12),
                quantity: row.get(13),
                ring_gauge_id: row.get(14),
                length: row.get(15),
                image_url: row.get(16),
                retail_link: row.get(17),
                is_active: row.get(18),
                created_at: row.get(19),
                updated_at: row.get(20),
            };
            Ok(warp::reply::json(&cigar))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(&json!({"error": "Failed to update cigar"})))
        }
    }
}

pub async fn delete_cigar(
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

    // CRITICAL: Verify the cigar belongs to the user or is shared with full permission (delete requires full)
    // Check if user has manage permission (only owner or full shared access can delete)
    let db_check = pool.get().await.map_err(|_e| {
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    let humidor_query = "SELECT humidor_id FROM cigars WHERE id = $1";
    let humidor_id: Uuid = match db_check.query_opt(humidor_query, &[&id]).await {
        Ok(Some(row)) => row.get(0),
        Ok(None) => {
            return Err(warp::reject::custom(AppError::NotFound(
                "Cigar not found".to_string(),
            )));
        }
        Err(e) => {
            return Err(warp::reject::custom(AppError::DatabaseError(format!(
                "Failed to find cigar: {}",
                e
            ))));
        }
    };

    // Check if user can manage (delete requires full permission)
    use crate::handlers::humidor_shares::can_manage_humidor;
    if !can_manage_humidor(&pool, &auth.user_id, &humidor_id)
        .await
        .map_err(warp::reject::custom)?
    {
        return Err(warp::reject::custom(AppError::Forbidden(
            "You do not have permission to delete cigars from this humidor".to_string(),
        )));
    }

    // Hard delete: actually remove the cigar from the database
    // Note: favorites will keep snapshot data due to ON DELETE SET NULL on cigar_id
    match db.execute("DELETE FROM cigars WHERE id = $1", &[&id]).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok(warp::reply::json(
                    &json!({"message": "Cigar deleted successfully"}),
                ))
            } else {
                Ok(warp::reply::json(&json!({"error": "Cigar not found"})))
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error");
            Ok(warp::reply::json(
                &json!({"error": "Failed to delete cigar"}),
            ))
        }
    }
}

#[derive(serde::Deserialize)]
pub struct TransferCigarRequest {
    destination_humidor_id: Uuid,
    quantity: i32,
}

pub async fn transfer_cigar(
    id: Uuid,
    transfer_req: TransferCigarRequest,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    // Validate quantity
    if transfer_req.quantity <= 0 {
        return Err(warp::reject::custom(AppError::ValidationError(
            "Quantity must be greater than 0".to_string(),
        )));
    }

    let mut db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    // Start a transaction for atomicity
    let transaction = db.transaction().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to start transaction");
        warp::reject::custom(AppError::DatabaseError(
            "Failed to start transaction".to_string(),
        ))
    })?;

    // CRITICAL: Verify the source cigar belongs to the user or is shared with edit permission
    verify_cigar_ownership(&pool, id, auth.user_id, true)
        .await
        .map_err(warp::reject::custom)?;

    // Verify destination humidor belongs to user or is shared with edit permission
    verify_humidor_ownership(
        &pool,
        Some(transfer_req.destination_humidor_id),
        auth.user_id,
        true,
    )
    .await
    .map_err(warp::reject::custom)?;

    // Get the source cigar (join with humidors to verify ownership)
    let source_cigar_row = transaction
        .query_one(
            "SELECT c.id, c.humidor_id, c.brand_id, c.name, c.size_id, c.strength_id, c.origin_id, 
                    c.wrapper, c.binder, c.filler, c.price, c.purchase_date, c.notes, c.quantity, 
                    c.ring_gauge_id, c.length, c.image_url, c.retail_link, c.is_active, h.user_id
             FROM cigars c
             INNER JOIN humidors h ON c.humidor_id = h.id
             WHERE c.id = $1 AND h.user_id = $2",
            &[&id, &auth.user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to fetch source cigar");
            warp::reject::custom(AppError::NotFound("Cigar not found".to_string()))
        })?;

    let source_humidor_id: Option<Uuid> = source_cigar_row.get(1);
    let current_quantity: i32 = source_cigar_row.get(13);

    // Verify quantity to transfer is valid
    if transfer_req.quantity > current_quantity {
        return Err(warp::reject::custom(AppError::ValidationError(format!(
            "Cannot transfer {} cigars. Only {} available.",
            transfer_req.quantity, current_quantity
        ))));
    }

    // Prevent transferring to the same humidor
    if source_humidor_id == Some(transfer_req.destination_humidor_id) {
        return Err(warp::reject::custom(AppError::ValidationError(
            "Cannot transfer to the same humidor".to_string(),
        )));
    }

    let now = Utc::now();

    // For now, skip checking for duplicates - just create a new cigar
    // This can be enhanced later to merge duplicates
    let existing_cigar: Option<tokio_postgres::Row> = None;

    if let Some(existing_row) = existing_cigar {
        // Cigar already exists in destination - just update quantity
        let existing_id: Uuid = existing_row.get(0);
        let existing_quantity: i32 = existing_row.get(1);
        let new_quantity = existing_quantity + transfer_req.quantity;

        transaction
            .execute(
                "UPDATE cigars SET quantity = $1, updated_at = $2 WHERE id = $3",
                &[&new_quantity, &now, &existing_id],
            )
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to update destination cigar quantity");
                warp::reject::custom(AppError::DatabaseError(
                    "Failed to update destination cigar".to_string(),
                ))
            })?;
    } else {
        // Create new cigar in destination humidor
        transaction
            .execute(
                "INSERT INTO cigars (id, humidor_id, brand_id, name, size_id, strength_id, origin_id, 
                                    wrapper, binder, filler, price, purchase_date, notes, quantity, 
                                    ring_gauge_id, length, image_url, retail_link, is_active, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)",
                &[
                    &Uuid::new_v4(),
                    &transfer_req.destination_humidor_id,
                    &source_cigar_row.get::<_, Option<Uuid>>(2),   // brand_id
                    &source_cigar_row.get::<_, String>(3),         // name
                    &source_cigar_row.get::<_, Option<Uuid>>(4),   // size_id
                    &source_cigar_row.get::<_, Option<Uuid>>(5),   // strength_id
                    &source_cigar_row.get::<_, Option<Uuid>>(6),   // origin_id
                    &source_cigar_row.get::<_, Option<String>>(7), // wrapper
                    &source_cigar_row.get::<_, Option<String>>(8), // binder
                    &source_cigar_row.get::<_, Option<String>>(9), // filler
                    &source_cigar_row.get::<_, Option<f64>>(10),   // price
                    &source_cigar_row.get::<_, Option<chrono::DateTime<Utc>>>(11), // purchase_date
                    &source_cigar_row.get::<_, Option<String>>(12), // notes
                    &transfer_req.quantity,
                    &source_cigar_row.get::<_, Option<Uuid>>(14),  // ring_gauge_id
                    &source_cigar_row.get::<_, Option<f64>>(15),   // length
                    &source_cigar_row.get::<_, Option<String>>(16), // image_url
                    &source_cigar_row.get::<_, Option<String>>(17), // retail_link
                    &true, // is_active
                    &now,
                    &now,
                ],
            )
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to create destination cigar");
                warp::reject::custom(AppError::DatabaseError(
                    "Failed to create destination cigar".to_string(),
                ))
            })?;
    }

    // Update or delete source cigar
    let new_source_quantity = current_quantity - transfer_req.quantity;
    if new_source_quantity == 0 {
        // Delete the source cigar if quantity reaches 0
        transaction
            .execute("DELETE FROM cigars WHERE id = $1", &[&id])
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to delete source cigar");
                warp::reject::custom(AppError::DatabaseError(
                    "Failed to remove source cigar".to_string(),
                ))
            })?;
    } else {
        // Update source cigar quantity
        transaction
            .execute(
                "UPDATE cigars SET quantity = $1, updated_at = $2 WHERE id = $3",
                &[&new_source_quantity, &now, &id],
            )
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to update source cigar quantity");
                warp::reject::custom(AppError::DatabaseError(
                    "Failed to update source cigar".to_string(),
                ))
            })?;
    }

    // Commit transaction
    transaction.commit().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to commit transaction");
        warp::reject::custom(AppError::DatabaseError(
            "Failed to commit transfer".to_string(),
        ))
    })?;

    Ok(warp::reply::json(&json!({
        "message": "Cigar transferred successfully",
        "transferred_quantity": transfer_req.quantity
    })))
}

#[derive(serde::Deserialize)]
pub struct ScrapeRequest {
    url: String,
}

pub async fn scrape_cigar_url(
    body: ScrapeRequest,
    _auth: AuthContext,
) -> Result<impl Reply, Rejection> {
    use crate::services::scrape_cigar_url;

    match scrape_cigar_url(&body.url).await {
        Ok(data) => Ok(warp::reply::json(&data)),
        Err(e) => {
            tracing::error!(error = %e, "Scraping error");
            Ok(warp::reply::json(
                &json!({"error": "Failed to scrape cigar information"}),
            ))
        }
    }
}

/// Get a random cigar recommendation
/// GET /api/v1/cigars/recommend?humidor_id={optional}
pub async fn get_random_cigar(
    params: std::collections::HashMap<String, String>,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    use crate::models::{CigarWithNames, RecommendCigarResponse};
    use warp::http::StatusCode;
    use warp::reply;

    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    let user_id = auth.user_id;

    // Parse optional humidor_id filter
    let humidor_id_filter = params
        .get("humidor_id")
        .and_then(|s| Uuid::parse_str(s).ok());

    // If humidor_id provided, verify access
    if let Some(hid) = humidor_id_filter {
        verify_humidor_ownership(&pool, Some(hid), user_id, false)
            .await
            .map_err(warp::reject::custom)?;
    }

    // Build query to get random cigar
    // IMPORTANT: Only select cigars with quantity > 0 and is_active = true
    let query = if humidor_id_filter.is_some() {
        // Single humidor
        "SELECT c.id, c.humidor_id, c.name, b.name as brand_name, s.name as size_name,
                st.name as strength_name, o.name as origin_name, c.wrapper, c.binder, c.filler,
                c.quantity, c.notes, c.purchase_date, c.price as purchase_price, c.retail_link,
                c.created_at, c.updated_at, c.is_active,
                rg.gauge as ring_gauge, c.length, st.level as strength_score, c.image_url,
                c.brand_id, c.size_id, c.strength_id, c.origin_id, c.ring_gauge_id
         FROM cigars c
         LEFT JOIN brands b ON c.brand_id = b.id
         LEFT JOIN sizes s ON c.size_id = s.id
         LEFT JOIN strengths st ON c.strength_id = st.id
         LEFT JOIN origins o ON c.origin_id = o.id
         LEFT JOIN ring_gauges rg ON c.ring_gauge_id = rg.id
         INNER JOIN humidors h ON c.humidor_id = h.id
         WHERE c.humidor_id = $1 
           AND c.quantity > 0 
           AND c.is_active = true
         ORDER BY RANDOM()
         LIMIT 1"
    } else {
        // All user's humidors + shared humidors
        "SELECT c.id, c.humidor_id, c.name, b.name as brand_name, s.name as size_name,
                st.name as strength_name, o.name as origin_name, c.wrapper, c.binder, c.filler,
                c.quantity, c.notes, c.purchase_date, c.price as purchase_price, c.retail_link,
                c.created_at, c.updated_at, c.is_active,
                rg.gauge as ring_gauge, c.length, st.level as strength_score, c.image_url,
                c.brand_id, c.size_id, c.strength_id, c.origin_id, c.ring_gauge_id
         FROM cigars c
         LEFT JOIN brands b ON c.brand_id = b.id
         LEFT JOIN sizes s ON c.size_id = s.id
         LEFT JOIN strengths st ON c.strength_id = st.id
         LEFT JOIN origins o ON c.origin_id = o.id
         LEFT JOIN ring_gauges rg ON c.ring_gauge_id = rg.id
         INNER JOIN humidors h ON c.humidor_id = h.id
         LEFT JOIN humidor_shares hs ON h.id = hs.humidor_id AND hs.shared_with_user_id = $1
         WHERE (h.user_id = $1 OR hs.id IS NOT NULL)
           AND c.quantity > 0 
           AND c.is_active = true
         ORDER BY RANDOM()
         LIMIT 1"
    };

    // Also get total count of eligible cigars
    let count_query = if humidor_id_filter.is_some() {
        "SELECT COUNT(*) FROM cigars c
         INNER JOIN humidors h ON c.humidor_id = h.id
         WHERE c.humidor_id = $1 
           AND c.quantity > 0 
           AND c.is_active = true"
    } else {
        "SELECT COUNT(*) FROM cigars c
         INNER JOIN humidors h ON c.humidor_id = h.id
         LEFT JOIN humidor_shares hs ON h.id = hs.humidor_id AND hs.shared_with_user_id = $1
         WHERE (h.user_id = $1 OR hs.id IS NOT NULL)
           AND c.quantity > 0 
           AND c.is_active = true"
    };

    // Execute queries
    let count_result = if let Some(hid) = humidor_id_filter {
        db.query_one(count_query, &[&hid]).await
    } else {
        db.query_one(count_query, &[&user_id]).await
    };

    let eligible_count: i64 = match count_result {
        Ok(row) => row.get(0),
        Err(e) => {
            tracing::error!(error = %e, "Failed to count eligible cigars");
            return Err(warp::reject::custom(AppError::DatabaseError(
                "Failed to count cigars".to_string(),
            )));
        }
    };

    // Execute random selection
    let cigar_result = if let Some(hid) = humidor_id_filter {
        db.query_opt(query, &[&hid]).await
    } else {
        db.query_opt(query, &[&user_id]).await
    };

    match cigar_result {
        Ok(Some(row)) => {
            // Extract cigar data from row with better error handling
            let cigar = CigarWithNames {
                cigar: Cigar {
                    id: row.get("id"),
                    humidor_id: row.get("humidor_id"),
                    brand_id: row.get("brand_id"),
                    name: row.get("name"),
                    size_id: row.get("size_id"),
                    strength_id: row.get("strength_id"),
                    origin_id: row.get("origin_id"),
                    wrapper: row.get("wrapper"),
                    binder: row.get("binder"),
                    filler: row.get("filler"),
                    price: row.get("purchase_price"),
                    purchase_date: row.get("purchase_date"),
                    notes: row.get("notes"),
                    quantity: row.get("quantity"),
                    ring_gauge_id: row.get("ring_gauge_id"),
                    length: row.get("length"),
                    image_url: row.get("image_url"),
                    retail_link: row.get("retail_link"),
                    is_active: row.get("is_active"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                },
                brand_name: row.get("brand_name"),
                size_name: row.get("size_name"),
                strength_name: row.get("strength_name"),
                origin_name: row.get("origin_name"),
                ring_gauge: row.get("ring_gauge"),
            };

            let message = if eligible_count > 1 {
                format!(
                    "How about this one? ({} other options available)",
                    eligible_count - 1
                )
            } else {
                "This is your only available cigar!".to_string()
            };

            Ok(reply::with_status(
                reply::json(&RecommendCigarResponse {
                    cigar: Some(cigar),
                    eligible_count,
                    message,
                }),
                StatusCode::OK,
            ))
        }
        Ok(None) => {
            // No cigars available
            Ok(reply::with_status(
                reply::json(&RecommendCigarResponse {
                    cigar: None,
                    eligible_count: 0,
                    message: "No cigars available for recommendation".to_string(),
                }),
                StatusCode::OK,
            ))
        }
        Err(e) => {
            tracing::error!(error = ?e, "Failed to get random cigar - database query error");
            Err(warp::reject::custom(AppError::DatabaseError(
                "Failed to get recommendation".to_string(),
            )))
        }
    }
}
