use crate::errors::AppError;
use crate::handlers::auth::seed_default_organizers;
use crate::middleware::AuthContext;
use crate::models::{
    AdminChangePasswordRequest, AdminCreateUserRequest, AdminToggleActiveRequest,
    AdminUpdateUserRequest, UserListResponse, UserResponse,
};
use crate::DbPool;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;
use warp::Reply;

// Authentication and JWT utilities
use bcrypt::{hash, DEFAULT_COST};

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
    if let Some(is_admin) = request.is_admin {
        if !is_admin && user_id == auth.user_id {
            return Err(warp::reject::custom(AppError::BadRequest(
                "Cannot demote yourself from admin".to_string(),
            )));
        }
    }

    // Check if trying to deactivate self
    if let Some(is_active) = request.is_active {
        if !is_active && user_id == auth.user_id {
            return Err(warp::reject::custom(AppError::BadRequest(
                "Cannot deactivate your own account".to_string(),
            )));
        }
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
