use crate::DbPool;
use crate::errors::AppError;
use crate::handlers::auth::seed_default_organizers;
use crate::middleware::AuthContext;
use crate::models::{
    AdminChangePasswordRequest, AdminCreateUserRequest, AdminToggleActiveRequest,
    AdminUpdateUserRequest, TransferOwnershipRequest, TransferOwnershipResponse, UserListResponse,
    UserResponse,
};
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;
use warp::Reply;

// Authentication and JWT utilities
use bcrypt::{DEFAULT_COST, hash};

// Async-safe bcrypt operation using tokio::task::spawn_blocking
async fn hash_password(password: String) -> Result<String, bcrypt::BcryptError> {
    tokio::task::spawn_blocking(move || hash(&password, DEFAULT_COST))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Task join error during password hashing");
            bcrypt::BcryptError::InvalidCost(DEFAULT_COST.to_string())
        })?
}

/// List all users with pagination
pub async fn list_users(
    auth: AuthContext,
    page: Option<i32>,
    per_page: Option<i32>,
    pool: DbPool,
) -> Result<impl Reply, warp::Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    let page = page.unwrap_or(1).max(1);
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    // Get total count
    let count_query = "SELECT COUNT(*) FROM users";
    let total: i64 = match db.query_one(count_query, &[]).await {
        Ok(row) => row.get(0),
        Err(e) => {
            tracing::error!(error = %e, "Failed to count users");
            return Err(warp::reject::custom(AppError::DatabaseError(
                "Failed to count users".to_string(),
            )));
        }
    };

    // Get paginated users
    let query = "
        SELECT id, username, email, full_name, is_admin, is_active, created_at, updated_at
        FROM users
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
    ";

    match db
        .query(query, &[&(per_page as i64), &(offset as i64)])
        .await
    {
        Ok(rows) => {
            let users: Vec<UserResponse> = rows
                .iter()
                .map(|row| UserResponse {
                    id: row.get("id"),
                    username: row.get("username"),
                    email: row.get("email"),
                    full_name: row.get("full_name"),
                    is_admin: row.get("is_admin"),
                    is_active: row.get("is_active"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                })
                .collect();

            let response = UserListResponse {
                users,
                total,
                page,
                per_page,
            };

            tracing::info!(
                admin_id = %auth.user_id,
                page = page,
                per_page = per_page,
                total = total,
                "Admin listed users"
            );

            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error listing users");
            Err(warp::reject::custom(AppError::DatabaseError(
                "Failed to list users".to_string(),
            )))
        }
    }
}

/// Get a specific user by ID
pub async fn get_user(
    user_id: Uuid,
    _auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, warp::Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    let query = "
        SELECT id, username, email, full_name, is_admin, is_active, created_at, updated_at
        FROM users
        WHERE id = $1
    ";

    match db.query_opt(query, &[&user_id]).await {
        Ok(Some(row)) => {
            let user = UserResponse {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                full_name: row.get("full_name"),
                is_admin: row.get("is_admin"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };

            Ok(warp::reply::json(&user))
        }
        Ok(None) => Err(warp::reject::custom(AppError::NotFound("User".to_string()))),
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "Database error fetching user");
            Err(warp::reject::custom(AppError::DatabaseError(
                "Failed to fetch user".to_string(),
            )))
        }
    }
}

/// Create a new user (admin only)
pub async fn create_user(
    request: AdminCreateUserRequest,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, warp::Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    // Hash password
    let password_hash = match hash_password(request.password.clone()).await {
        Ok(hash) => hash,
        Err(e) => {
            tracing::error!(error = %e, "Password hashing error");
            return Err(warp::reject::custom(AppError::InternalServerError(
                "Failed to process password".to_string(),
            )));
        }
    };

    let user_id = Uuid::new_v4();
    let now = Utc::now();

    let query = "
        INSERT INTO users (id, username, email, full_name, password_hash, is_admin, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, username, email, full_name, is_admin, is_active, created_at, updated_at
    ";

    match db
        .query_one(
            query,
            &[
                &user_id,
                &request.username,
                &request.email,
                &request.full_name,
                &password_hash,
                &request.is_admin,
                &request.is_active,
                &now,
                &now,
            ],
        )
        .await
    {
        Ok(row) => {
            let user = UserResponse {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                full_name: row.get("full_name"),
                is_admin: row.get("is_admin"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };

            // Seed default organizers for the new user
            if let Err(e) = seed_default_organizers(&db, &user_id).await {
                tracing::error!(error = %e, user_id = %user_id, "Failed to seed default organizers for new user");
                // Don't fail user creation if organizer seeding fails
            }

            tracing::info!(
                admin_id = %auth.user_id,
                new_user_id = %user.id,
                username = %user.username,
                is_admin = user.is_admin,
                "Admin created new user"
            );

            Ok(warp::reply::with_status(
                warp::reply::json(&user),
                warp::http::StatusCode::CREATED,
            ))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database error creating user");
            if e.to_string().contains("duplicate key") {
                Err(warp::reject::custom(AppError::Conflict(
                    "Username or email already exists".to_string(),
                )))
            } else {
                Err(warp::reject::custom(AppError::DatabaseError(
                    "Failed to create user".to_string(),
                )))
            }
        }
    }
}

/// Update a user (admin only)
pub async fn update_user(
    user_id: Uuid,
    request: AdminUpdateUserRequest,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, warp::Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    // Check if trying to demote self
    if let Some(is_admin) = request.is_admin
        && !is_admin
        && user_id == auth.user_id
    {
        return Err(warp::reject::custom(AppError::BadRequest(
            "Cannot demote yourself from admin".to_string(),
        )));
    }

    // Check if trying to deactivate self
    if let Some(is_active) = request.is_active
        && !is_active
        && user_id == auth.user_id
    {
        return Err(warp::reject::custom(AppError::BadRequest(
            "Cannot deactivate your own account".to_string(),
        )));
    }

    let mut updates = Vec::new();
    let mut param_index = 1;

    if let Some(ref username) = request.username {
        updates.push((format!("username = ${}", param_index), username.clone()));
        param_index += 1;
    }

    if let Some(ref email) = request.email {
        updates.push((format!("email = ${}", param_index), email.clone()));
        param_index += 1;
    }

    if let Some(ref full_name) = request.full_name {
        updates.push((format!("full_name = ${}", param_index), full_name.clone()));
        param_index += 1;
    }

    if let Some(is_admin) = request.is_admin {
        // Check if this would remove the last admin
        if !is_admin {
            let admin_check = db
                .query_one(
                    "SELECT COUNT(*) FROM users WHERE is_admin = true AND id != $1",
                    &[&user_id],
                )
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "Failed to check admin count");
                    warp::reject::custom(AppError::DatabaseError(
                        "Failed to verify admin count".to_string(),
                    ))
                })?;

            let admin_count: i64 = admin_check.get(0);
            if admin_count == 0 {
                return Err(warp::reject::custom(AppError::BadRequest(
                    "Cannot remove last admin user".to_string(),
                )));
            }
        }

        updates.push((format!("is_admin = ${}", param_index), is_admin.to_string()));
        param_index += 1;
    }

    if let Some(is_active) = request.is_active {
        updates.push((
            format!("is_active = ${}", param_index),
            is_active.to_string(),
        ));
        param_index += 1;
    }

    if updates.is_empty() {
        return Err(warp::reject::custom(AppError::BadRequest(
            "No fields to update".to_string(),
        )));
    }

    // Build the update query dynamically
    let update_fields: Vec<String> = updates.iter().map(|(field, _)| field.clone()).collect();
    let query = format!(
        "UPDATE users SET {}, updated_at = NOW() WHERE id = ${}
         RETURNING id, username, email, full_name, is_admin, is_active, created_at, updated_at",
        update_fields.join(", "),
        param_index
    );

    // Execute with proper parameters
    let stmt = db.prepare(&query).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to prepare statement");
        warp::reject::custom(AppError::DatabaseError(
            "Failed to prepare query".to_string(),
        ))
    })?;

    // Build params dynamically based on what we're updating
    let mut query_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

    if let Some(ref username) = request.username {
        query_params.push(username);
    }
    if let Some(ref email) = request.email {
        query_params.push(email);
    }
    if let Some(ref full_name) = request.full_name {
        query_params.push(full_name);
    }
    if let Some(ref is_admin) = request.is_admin {
        query_params.push(is_admin);
    }
    if let Some(ref is_active) = request.is_active {
        query_params.push(is_active);
    }
    query_params.push(&user_id);

    match db.query_one(&stmt, &query_params).await {
        Ok(row) => {
            let user = UserResponse {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                full_name: row.get("full_name"),
                is_admin: row.get("is_admin"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };

            tracing::info!(
                admin_id = %auth.user_id,
                updated_user_id = %user.id,
                "Admin updated user"
            );

            Ok(warp::reply::json(&user))
        }
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "Database error updating user");
            if e.to_string().contains("duplicate key") {
                Err(warp::reject::custom(AppError::Conflict(
                    "Email already exists".to_string(),
                )))
            } else {
                Err(warp::reject::custom(AppError::DatabaseError(
                    "Failed to update user".to_string(),
                )))
            }
        }
    }
}

/// Delete a user (admin only)
pub async fn delete_user(
    user_id: Uuid,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, warp::Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    // Prevent deleting self
    if user_id == auth.user_id {
        return Err(warp::reject::custom(AppError::BadRequest(
            "Cannot delete your own account".to_string(),
        )));
    }

    // Check if user is an admin
    let user_check = db
        .query_one("SELECT is_admin FROM users WHERE id = $1", &[&user_id])
        .await
        .map_err(|e| {
            if e.to_string().contains("no rows") {
                warp::reject::custom(AppError::NotFound("User".to_string()))
            } else {
                tracing::error!(error = %e, "Failed to fetch user");
                warp::reject::custom(AppError::DatabaseError("Failed to fetch user".to_string()))
            }
        })?;

    let is_admin: bool = user_check.get(0);

    // If deleting an admin, check if it's the last one
    if is_admin {
        let admin_count_check = db
            .query_one("SELECT COUNT(*) FROM users WHERE is_admin = true", &[])
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to check admin count");
                warp::reject::custom(AppError::DatabaseError(
                    "Failed to verify admin count".to_string(),
                ))
            })?;

        let admin_count: i64 = admin_count_check.get(0);
        if admin_count <= 1 {
            return Err(warp::reject::custom(AppError::BadRequest(
                "Cannot delete the last admin user".to_string(),
            )));
        }
    }

    // Delete user (cascade will handle related data)
    match db
        .execute("DELETE FROM users WHERE id = $1", &[&user_id])
        .await
    {
        Ok(deleted) => {
            if deleted == 0 {
                return Err(warp::reject::custom(AppError::NotFound("User".to_string())));
            }

            tracing::info!(
                admin_id = %auth.user_id,
                deleted_user_id = %user_id,
                "Admin deleted user"
            );

            Ok(warp::reply::json(&json!({
                "message": "User deleted successfully"
            })))
        }
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "Database error deleting user");
            Err(warp::reject::custom(AppError::DatabaseError(
                "Failed to delete user".to_string(),
            )))
        }
    }
}

/// Toggle user active status (admin only)
pub async fn toggle_active(
    user_id: Uuid,
    request: AdminToggleActiveRequest,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, warp::Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    // Prevent deactivating self
    if user_id == auth.user_id && !request.is_active {
        return Err(warp::reject::custom(AppError::BadRequest(
            "Cannot deactivate your own account".to_string(),
        )));
    }

    let now = Utc::now();

    match db
        .execute(
            "UPDATE users SET is_active = $1, updated_at = $2 WHERE id = $3",
            &[&request.is_active, &now, &user_id],
        )
        .await
    {
        Ok(updated) => {
            if updated == 0 {
                return Err(warp::reject::custom(AppError::NotFound("User".to_string())));
            }

            tracing::info!(
                admin_id = %auth.user_id,
                target_user_id = %user_id,
                is_active = request.is_active,
                "Admin toggled user active status"
            );

            Ok(warp::reply::json(&serde_json::json!({
                "message": if request.is_active { "User activated successfully" } else { "User deactivated successfully" }
            })))
        }
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "Database error toggling user active status");
            Err(warp::reject::custom(AppError::DatabaseError(
                "Failed to toggle user active status".to_string(),
            )))
        }
    }
}

/// Change a user's password (admin only)
pub async fn change_user_password(
    user_id: Uuid,
    request: AdminChangePasswordRequest,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, warp::Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    // Hash new password
    let password_hash = match hash_password(request.new_password.clone()).await {
        Ok(hash) => hash,
        Err(e) => {
            tracing::error!(error = %e, "Password hashing error");
            return Err(warp::reject::custom(AppError::InternalServerError(
                "Failed to process password".to_string(),
            )));
        }
    };

    let now = Utc::now();

    match db
        .execute(
            "UPDATE users SET password_hash = $1, updated_at = $2 WHERE id = $3",
            &[&password_hash, &now, &user_id],
        )
        .await
    {
        Ok(updated) => {
            if updated == 0 {
                return Err(warp::reject::custom(AppError::NotFound("User".to_string())));
            }

            tracing::info!(
                admin_id = %auth.user_id,
                target_user_id = %user_id,
                "Admin changed user password"
            );

            Ok(warp::reply::json(&json!({
                "message": "Password changed successfully"
            })))
        }
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "Database error changing password");
            Err(warp::reject::custom(AppError::DatabaseError(
                "Failed to change password".to_string(),
            )))
        }
    }
}

/// Transfer ownership of all humidors and cigars from one user to another
pub async fn transfer_ownership(
    request: TransferOwnershipRequest,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, warp::Rejection> {
    // Validate that source and target users are different
    if request.from_user_id == request.to_user_id {
        return Err(warp::reject::custom(AppError::ValidationError(
            "Source and target users must be different".to_string(),
        )));
    }

    let mut db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    // Start transaction for atomic operation
    let transaction = db.transaction().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to start transaction");
        warp::reject::custom(AppError::DatabaseError(
            "Failed to start transaction".to_string(),
        ))
    })?;

    // Verify both users exist
    let verify_users_query = "
        SELECT id FROM users WHERE id = $1
        UNION ALL
        SELECT id FROM users WHERE id = $2
    ";

    let user_rows = transaction
        .query(
            verify_users_query,
            &[&request.from_user_id, &request.to_user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to verify users");
            warp::reject::custom(AppError::DatabaseError(
                "Failed to verify users".to_string(),
            ))
        })?;

    if user_rows.len() != 2 {
        return Err(warp::reject::custom(AppError::NotFound(
            "One or both users not found".to_string(),
        )));
    }

    // Determine which humidors to transfer
    let (count_cigars_query, transfer_humidors_query, delete_shares_query, query_params) =
        if let Some(humidor_id) = request.humidor_id {
            // Transfer single humidor
            // Verify the humidor belongs to the from_user
            let verify_query = "SELECT id FROM humidors WHERE id = $1 AND user_id = $2";
            let verify_result = transaction
                .query_opt(verify_query, &[&humidor_id, &request.from_user_id])
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "Failed to verify humidor ownership");
                    warp::reject::custom(AppError::DatabaseError(
                        "Failed to verify humidor ownership".to_string(),
                    ))
                })?;

            if verify_result.is_none() {
                return Err(warp::reject::custom(AppError::NotFound(
                    "Humidor not found or does not belong to source user".to_string(),
                )));
            }

            (
                "SELECT COUNT(*) FROM cigars WHERE humidor_id = $1",
                "UPDATE humidors SET user_id = $1, updated_at = NOW() WHERE id = $2 AND user_id = $3",
                "DELETE FROM humidor_shares WHERE humidor_id = $1",
                vec![humidor_id],
            )
        } else {
            // Transfer all humidors
            (
                "SELECT COUNT(*) FROM cigars c INNER JOIN humidors h ON c.humidor_id = h.id WHERE h.user_id = $1",
                "UPDATE humidors SET user_id = $1, updated_at = NOW() WHERE user_id = $2",
                "DELETE FROM humidor_shares WHERE humidor_id IN (SELECT id FROM humidors WHERE user_id = $1)",
                vec![],
            )
        };

    // Count cigars before transfer
    let cigar_count_row = if request.humidor_id.is_some() {
        transaction
            .query_one(count_cigars_query, &[&query_params[0]])
            .await
    } else {
        transaction
            .query_one(count_cigars_query, &[&request.from_user_id])
            .await
    }
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to count cigars");
        warp::reject::custom(AppError::DatabaseError(
            "Failed to count cigars".to_string(),
        ))
    })?;

    let cigars_transferred: i64 = cigar_count_row.get(0);

    // Transfer humidors
    let humidors_transferred = if let Some(humidor_id) = request.humidor_id {
        transaction
            .execute(
                transfer_humidors_query,
                &[&request.to_user_id, &humidor_id, &request.from_user_id],
            )
            .await
    } else {
        transaction
            .execute(
                transfer_humidors_query,
                &[&request.to_user_id, &request.from_user_id],
            )
            .await
    }
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to transfer humidors");
        warp::reject::custom(AppError::DatabaseError(
            "Failed to transfer humidors".to_string(),
        ))
    })?;

    // Delete humidor shares
    if let Some(humidor_id) = request.humidor_id {
        transaction
            .execute(delete_shares_query, &[&humidor_id])
            .await
    } else {
        transaction
            .execute(delete_shares_query, &[&request.to_user_id])
            .await
    }
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to clean up humidor shares");
        warp::reject::custom(AppError::DatabaseError(
            "Failed to clean up humidor shares".to_string(),
        ))
    })?;

    // Map organizer data for cigars being transferred
    // For each organizer type, create new records for target user if needed
    // and update cigar foreign keys to reference the new organizer IDs

    // After humidors are transferred, they belong to to_user_id
    // We need to find cigars in those humidors and update their organizer references
    let humidor_filter = if let Some(humidor_id) = request.humidor_id {
        format!("humidor_id = '{}'", humidor_id)
    } else {
        // All humidors that were just transferred (now owned by to_user_id)
        format!(
            "humidor_id IN (SELECT id FROM humidors WHERE user_id = '{}')",
            request.to_user_id
        )
    };

    // Copy organizer data (brands)
    // Step 1: Insert brands for target user (skip if already exist)
    let insert_brands_query = format!(
        "INSERT INTO brands (user_id, name, country, website, description, created_at, updated_at)
        SELECT DISTINCT $1::uuid, b.name, b.country, b.website, b.description, NOW(), NOW()
        FROM cigars c
        INNER JOIN brands b ON c.brand_id = b.id
        WHERE c.{} AND b.user_id = $2::uuid AND b.name IS NOT NULL
        ON CONFLICT (user_id, name) DO NOTHING",
        humidor_filter
    );

    let brands_inserted = transaction
        .execute(
            &insert_brands_query,
            &[&request.to_user_id, &request.from_user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, query = %insert_brands_query, "Failed to insert brands");
            warp::reject::custom(AppError::DatabaseError(format!(
                "Failed to copy brands: {:?}",
                e
            )))
        })?;

    tracing::debug!(brands_inserted, "Brands inserted for target user");

    // Step 2: Update cigars to reference target user's brands
    let update_brands_query = format!(
        "UPDATE cigars c
        SET brand_id = target_b.id
        FROM brands old_b
        JOIN brands target_b ON target_b.name = old_b.name AND target_b.user_id = $1::uuid
        WHERE c.{} AND c.brand_id = old_b.id AND old_b.user_id = $2::uuid",
        humidor_filter
    );

    transaction
        .execute(
            &update_brands_query,
            &[&request.to_user_id, &request.from_user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to update brand references");
            warp::reject::custom(AppError::DatabaseError(
                "Failed to copy organizer data".to_string(),
            ))
        })?;

    // Map origins - Step 1: Insert origins for target user
    let insert_origins_query = format!(
        "INSERT INTO origins (user_id, name, country, region, description, created_at, updated_at)
        SELECT DISTINCT $1::uuid, o.name, o.country, o.region, o.description, NOW(), NOW()
        FROM cigars c
        INNER JOIN origins o ON c.origin_id = o.id
        WHERE c.{} AND o.user_id = $2::uuid AND o.name IS NOT NULL
        ON CONFLICT (user_id, name) DO NOTHING",
        humidor_filter
    );

    transaction
        .execute(
            &insert_origins_query,
            &[&request.to_user_id, &request.from_user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, query = %insert_origins_query, "Failed to insert origins");
            warp::reject::custom(AppError::DatabaseError(format!(
                "Failed to copy origins: {:?}",
                e
            )))
        })?;

    // Map origins - Step 2: Update cigar foreign keys
    let update_origins_query = format!(
        "UPDATE cigars c
        SET origin_id = target_o.id
        FROM origins old_o
        INNER JOIN origins target_o ON target_o.name = old_o.name AND target_o.user_id = $1::uuid
        WHERE c.{} AND c.origin_id = old_o.id AND old_o.user_id = $2::uuid",
        humidor_filter
    );

    transaction
        .execute(
            &update_origins_query,
            &[&request.to_user_id, &request.from_user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to map origins");
            warp::reject::custom(AppError::DatabaseError(
                "Failed to copy organizer data".to_string(),
            ))
        })?;

    // Map strengths - Step 1: Insert strengths for target user
    let insert_strengths_query = format!(
        "INSERT INTO strengths (user_id, name, level, description, created_at, updated_at)
        SELECT DISTINCT $1::uuid, s.name, s.level, s.description, NOW(), NOW()
        FROM cigars c
        INNER JOIN strengths s ON c.strength_id = s.id
        WHERE c.{} AND s.user_id = $2::uuid AND s.name IS NOT NULL
        ON CONFLICT (user_id, name) DO NOTHING",
        humidor_filter
    );

    transaction
        .execute(&insert_strengths_query, &[&request.to_user_id, &request.from_user_id])
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, query = %insert_strengths_query, "Failed to insert strengths");
            warp::reject::custom(AppError::DatabaseError(
                format!("Failed to copy strengths: {:?}", e)
            ))
        })?;

    // Map strengths - Step 2: Update cigar foreign keys
    let update_strengths_query = format!(
        "UPDATE cigars c
        SET strength_id = target_s.id
        FROM strengths old_s
        INNER JOIN strengths target_s ON target_s.name = old_s.name AND target_s.user_id = $1::uuid
        WHERE c.{} AND c.strength_id = old_s.id AND old_s.user_id = $2::uuid",
        humidor_filter
    );

    transaction
        .execute(
            &update_strengths_query,
            &[&request.to_user_id, &request.from_user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to map strengths");
            warp::reject::custom(AppError::DatabaseError(
                "Failed to copy organizer data".to_string(),
            ))
        })?;

    // Map sizes - Step 1: Insert sizes for target user
    let insert_sizes_query = format!(
        "INSERT INTO sizes (user_id, name, length_inches, ring_gauge, description, created_at, updated_at)
        SELECT DISTINCT $1::uuid, sz.name, sz.length_inches, sz.ring_gauge, sz.description, NOW(), NOW()
        FROM cigars c
        INNER JOIN sizes sz ON c.size_id = sz.id
        WHERE c.{} AND sz.user_id = $2::uuid AND sz.name IS NOT NULL
        ON CONFLICT (user_id, name) DO NOTHING",
        humidor_filter
    );

    transaction
        .execute(
            &insert_sizes_query,
            &[&request.to_user_id, &request.from_user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, query = %insert_sizes_query, "Failed to insert sizes");
            warp::reject::custom(AppError::DatabaseError(format!(
                "Failed to copy sizes: {:?}",
                e
            )))
        })?;

    // Map sizes - Step 2: Update cigar foreign keys
    let update_sizes_query = format!(
        "UPDATE cigars c
        SET size_id = target_sz.id
        FROM sizes old_sz
        INNER JOIN sizes target_sz ON target_sz.name = old_sz.name AND target_sz.user_id = $1::uuid
        WHERE c.{} AND c.size_id = old_sz.id AND old_sz.user_id = $2::uuid",
        humidor_filter
    );

    transaction
        .execute(
            &update_sizes_query,
            &[&request.to_user_id, &request.from_user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to map sizes");
            warp::reject::custom(AppError::DatabaseError(
                "Failed to copy organizer data".to_string(),
            ))
        })?;

    // Map ring_gauges - Step 1: Insert ring_gauges for target user
    let insert_ring_gauges_query = format!(
        "INSERT INTO ring_gauges (user_id, gauge, description, common_names, created_at, updated_at)
        SELECT DISTINCT $1::uuid, rg.gauge, rg.description, rg.common_names, NOW(), NOW()
        FROM cigars c
        INNER JOIN ring_gauges rg ON c.ring_gauge_id = rg.id
        WHERE c.{} AND rg.user_id = $2::uuid
        ON CONFLICT (user_id, gauge) DO NOTHING",
        humidor_filter
    );

    transaction
        .execute(&insert_ring_gauges_query, &[&request.to_user_id, &request.from_user_id])
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, query = %insert_ring_gauges_query, "Failed to insert ring_gauges");
            warp::reject::custom(AppError::DatabaseError(
                format!("Failed to copy ring_gauges: {:?}", e)
            ))
        })?;

    // Map ring_gauges - Step 2: Update cigar foreign keys
    let update_ring_gauges_query = format!(
        "UPDATE cigars c
        SET ring_gauge_id = target_rg.id
        FROM ring_gauges old_rg
        INNER JOIN ring_gauges target_rg ON target_rg.gauge = old_rg.gauge AND target_rg.user_id = $1::uuid
        WHERE c.{} AND c.ring_gauge_id = old_rg.id AND old_rg.user_id = $2::uuid",
        humidor_filter
    );

    transaction
        .execute(
            &update_ring_gauges_query,
            &[&request.to_user_id, &request.from_user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to map ring_gauges");
            warp::reject::custom(AppError::DatabaseError(
                "Failed to copy organizer data".to_string(),
            ))
        })?;

    // Commit transaction
    transaction.commit().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to commit transaction");
        warp::reject::custom(AppError::DatabaseError(
            "Failed to commit ownership transfer".to_string(),
        ))
    })?;

    tracing::info!(
        admin_id = %auth.user_id,
        from_user_id = %request.from_user_id,
        to_user_id = %request.to_user_id,
        humidors_transferred = humidors_transferred,
        cigars_transferred = cigars_transferred,
        "Ownership transferred successfully"
    );

    Ok(warp::reply::json(&TransferOwnershipResponse {
        humidors_transferred: humidors_transferred as i64,
        cigars_transferred,
    }))
}

/// Get humidors for a specific user (admin only)
pub async fn get_user_humidors(
    user_id: Uuid,
    _auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, warp::Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::custom(AppError::DatabaseError(
            "Database connection failed".to_string(),
        ))
    })?;

    // Verify user exists
    let user_check_query = "SELECT id FROM users WHERE id = $1";
    let user_exists = db
        .query_opt(user_check_query, &[&user_id])
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to verify user");
            warp::reject::custom(AppError::DatabaseError("Failed to verify user".to_string()))
        })?;

    if user_exists.is_none() {
        return Err(warp::reject::custom(AppError::NotFound(
            "User not found".to_string(),
        )));
    }

    // Get humidors owned by user with cigar count
    let query = "
        SELECT 
            h.id, 
            h.user_id, 
            h.name, 
            h.description, 
            h.capacity, 
            h.target_humidity, 
            h.location, 
            h.image_url, 
            h.created_at, 
            h.updated_at,
            COUNT(c.id) as cigar_count
        FROM humidors h
        LEFT JOIN cigars c ON h.id = c.humidor_id
        WHERE h.user_id = $1
        GROUP BY h.id, h.user_id, h.name, h.description, h.capacity, h.target_humidity, h.location, h.image_url, h.created_at, h.updated_at
        ORDER BY h.created_at ASC
    ";

    match db.query(query, &[&user_id]).await {
        Ok(rows) => {
            let humidors: Vec<serde_json::Value> = rows
                .iter()
                .map(|row| {
                    json!({
                        "id": row.get::<_, Uuid>("id"),
                        "user_id": row.get::<_, Uuid>("user_id"),
                        "name": row.get::<_, String>("name"),
                        "description": row.get::<_, Option<String>>("description"),
                        "capacity": row.get::<_, Option<i32>>("capacity"),
                        "target_humidity": row.get::<_, Option<i32>>("target_humidity"),
                        "location": row.get::<_, Option<String>>("location"),
                        "image_url": row.get::<_, Option<String>>("image_url"),
                        "cigar_count": row.get::<_, i64>("cigar_count"),
                        "created_at": row.get::<_, chrono::DateTime<Utc>>("created_at"),
                        "updated_at": row.get::<_, chrono::DateTime<Utc>>("updated_at")
                    })
                })
                .collect();

            Ok(warp::reply::json(&humidors))
        }
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "Failed to fetch humidors");
            Err(warp::reject::custom(AppError::DatabaseError(
                "Failed to fetch humidors".to_string(),
            )))
        }
    }
}
