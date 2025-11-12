use crate::errors::AppError;
use crate::middleware::auth::AuthContext;
use crate::models::{
    HumidorShareResponse, HumidorSharesListResponse, PermissionLevel, ShareHumidorRequest,
    SharedHumidorInfo, SharedHumidorsResponse, UpdateSharePermissionRequest, UserInfo,
};
use deadpool_postgres::Pool;
use uuid::Uuid;
use warp::{reject, reply, Rejection, Reply};

/// Helper function to get user's permission level for a humidor
/// Returns None if user has no access (not owner, not shared)
pub async fn get_user_permission_level(
    pool: &Pool,
    user_id: &Uuid,
    humidor_id: &Uuid,
) -> Result<Option<PermissionLevel>, AppError> {
    let client = pool.get().await.map_err(|e| {
        tracing::error!("Failed to get database connection: {}", e);
        AppError::DatabaseError("Failed to connect to database".to_string())
    })?;

    // Check if user is the owner
    let owner_check = client
        .query_opt(
            "SELECT id FROM humidors WHERE id = $1 AND user_id = $2",
            &[humidor_id, user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to check humidor ownership: {}", e);
            AppError::DatabaseError("Failed to check ownership".to_string())
        })?;

    if owner_check.is_some() {
        // Owner has full permissions
        return Ok(Some(PermissionLevel::Full));
    }

    // Check if humidor is shared with user
    let share_result = client
        .query_opt(
            "SELECT permission_level FROM humidor_shares 
             WHERE humidor_id = $1 AND shared_with_user_id = $2",
            &[humidor_id, user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to check humidor share: {}", e);
            AppError::DatabaseError("Failed to check share permission".to_string())
        })?;

    if let Some(row) = share_result {
        let permission_str: String = row.get(0);
        let permission = PermissionLevel::from_str(&permission_str)
            .map_err(|e| AppError::ValidationError(e))?;
        return Ok(Some(permission));
    }

    Ok(None)
}

/// Helper function to check if user can view a humidor
pub async fn can_view_humidor(
    pool: &Pool,
    user_id: &Uuid,
    humidor_id: &Uuid,
) -> Result<bool, AppError> {
    let permission = get_user_permission_level(pool, user_id, humidor_id).await?;
    Ok(permission.map_or(false, |p| p.can_view()))
}

/// Helper function to check if user can edit a humidor (add/update cigars)
pub async fn can_edit_humidor(
    pool: &Pool,
    user_id: &Uuid,
    humidor_id: &Uuid,
) -> Result<bool, AppError> {
    let permission = get_user_permission_level(pool, user_id, humidor_id).await?;
    Ok(permission.map_or(false, |p| p.can_edit()))
}

/// Helper function to check if user can manage a humidor (delete cigars, manage shares)
pub async fn can_manage_humidor(
    pool: &Pool,
    user_id: &Uuid,
    humidor_id: &Uuid,
) -> Result<bool, AppError> {
    let permission = get_user_permission_level(pool, user_id, humidor_id).await?;
    Ok(permission.map_or(false, |p| p.can_manage()))
}

/// Helper function to check if user is the owner of a humidor
pub async fn is_humidor_owner(
    pool: &Pool,
    user_id: &Uuid,
    humidor_id: &Uuid,
) -> Result<bool, AppError> {
    let client = pool.get().await.map_err(|e| {
        tracing::error!("Failed to get database connection: {}", e);
        AppError::DatabaseError("Failed to connect to database".to_string())
    })?;

    let result = client
        .query_opt(
            "SELECT id FROM humidors WHERE id = $1 AND user_id = $2",
            &[humidor_id, user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to check humidor ownership: {}", e);
            AppError::DatabaseError("Failed to check ownership".to_string())
        })?;

    Ok(result.is_some())
}

/// Share a humidor with another user
/// POST /api/v1/humidors/:id/share
pub async fn share_humidor(
    humidor_id: Uuid,
    auth: AuthContext,
    request: ShareHumidorRequest,
    pool: Pool,
) -> Result<impl Reply, Rejection> {
    tracing::info!(
        "User {} sharing humidor {} with user {} (permission: {:?})",
        auth.user_id,
        humidor_id,
        request.user_id,
        request.permission_level
    );

    // Verify user is the owner of the humidor
    if !is_humidor_owner(&pool, &auth.user_id, &humidor_id)
        .await
        .map_err(reject::custom)?
    {
        tracing::warn!(
            "User {} attempted to share humidor {} they don't own",
            auth.user_id,
            humidor_id
        );
        return Err(reject::custom(AppError::Forbidden(
            "You do not have permission to share this humidor".to_string(),
        )));
    }

    // Cannot share with yourself
    if request.user_id == auth.user_id {
        return Err(reject::custom(AppError::ValidationError(
            "Cannot share humidor with yourself".to_string(),
        )));
    }

    let client = pool.get().await.map_err(|e| {
        tracing::error!("Failed to get database connection: {}", e);
        reject::custom(AppError::DatabaseError(
            "Failed to connect to database".to_string(),
        ))
    })?;

    // Verify the target user exists and is active
    let user_exists = client
        .query_opt(
            "SELECT id FROM users WHERE id = $1 AND is_active = true",
            &[&request.user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to check user existence: {}", e);
            reject::custom(AppError::DatabaseError(
                "Failed to verify user".to_string(),
            ))
        })?;

    if user_exists.is_none() {
        return Err(reject::custom(AppError::NotFound(
            "User not found or inactive".to_string(),
        )));
    }

    // Insert or update the share
    let share_id = Uuid::new_v4();
    let permission_str = request.permission_level.as_str();

    client
        .execute(
            "INSERT INTO humidor_shares (id, humidor_id, shared_with_user_id, shared_by_user_id, permission_level)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (humidor_id, shared_with_user_id) 
             DO UPDATE SET permission_level = $5, updated_at = NOW()",
            &[
                &share_id,
                &humidor_id,
                &request.user_id,
                &auth.user_id,
                &permission_str,
            ],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create humidor share: {}", e);
            reject::custom(AppError::DatabaseError(
                "Failed to share humidor".to_string(),
            ))
        })?;

    tracing::info!(
        "Successfully shared humidor {} with user {}",
        humidor_id,
        request.user_id
    );

    Ok(reply::with_status(
        reply::json(&serde_json::json!({
            "message": "Humidor shared successfully",
            "share_id": share_id
        })),
        warp::http::StatusCode::CREATED,
    ))
}

/// Revoke access to a shared humidor
/// DELETE /api/v1/humidors/:id/share/:user_id
pub async fn revoke_share(
    humidor_id: Uuid,
    user_id: Uuid,
    auth: AuthContext,
    pool: Pool,
) -> Result<impl Reply, Rejection> {
    tracing::info!(
        "User {} revoking access to humidor {} from user {}",
        auth.user_id,
        humidor_id,
        user_id
    );

    // Verify user is the owner of the humidor
    if !is_humidor_owner(&pool, &auth.user_id, &humidor_id)
        .await
        .map_err(reject::custom)?
    {
        tracing::warn!(
            "User {} attempted to revoke share for humidor {} they don't own",
            auth.user_id,
            humidor_id
        );
        return Err(reject::custom(AppError::Forbidden(
            "You do not have permission to manage shares for this humidor".to_string(),
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
            "DELETE FROM humidor_shares WHERE humidor_id = $1 AND shared_with_user_id = $2",
            &[&humidor_id, &user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to revoke humidor share: {}", e);
            reject::custom(AppError::DatabaseError(
                "Failed to revoke share".to_string(),
            ))
        })?;

    if rows_affected == 0 {
        return Err(reject::custom(AppError::NotFound(
            "Share not found".to_string(),
        )));
    }

    tracing::info!(
        "Successfully revoked access to humidor {} from user {}",
        humidor_id,
        user_id
    );

    Ok(reply::with_status(
        reply::json(&serde_json::json!({
            "message": "Share revoked successfully"
        })),
        warp::http::StatusCode::OK,
    ))
}

/// Update permission level for a shared humidor
/// PATCH /api/v1/humidors/:id/share/:user_id
pub async fn update_share_permission(
    humidor_id: Uuid,
    user_id: Uuid,
    auth: AuthContext,
    request: UpdateSharePermissionRequest,
    pool: Pool,
) -> Result<impl Reply, Rejection> {
    tracing::info!(
        "User {} updating share permission for humidor {} user {} to {:?}",
        auth.user_id,
        humidor_id,
        user_id,
        request.permission_level
    );

    // Verify user is the owner of the humidor
    if !is_humidor_owner(&pool, &auth.user_id, &humidor_id)
        .await
        .map_err(reject::custom)?
    {
        tracing::warn!(
            "User {} attempted to update share for humidor {} they don't own",
            auth.user_id,
            humidor_id
        );
        return Err(reject::custom(AppError::Forbidden(
            "You do not have permission to manage shares for this humidor".to_string(),
        )));
    }

    let client = pool.get().await.map_err(|e| {
        tracing::error!("Failed to get database connection: {}", e);
        reject::custom(AppError::DatabaseError(
            "Failed to connect to database".to_string(),
        ))
    })?;

    let permission_str = request.permission_level.as_str();
    let rows_affected = client
        .execute(
            "UPDATE humidor_shares SET permission_level = $1, updated_at = NOW() 
             WHERE humidor_id = $2 AND shared_with_user_id = $3",
            &[&permission_str, &humidor_id, &user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to update share permission: {}", e);
            reject::custom(AppError::DatabaseError(
                "Failed to update permission".to_string(),
            ))
        })?;

    if rows_affected == 0 {
        return Err(reject::custom(AppError::NotFound(
            "Share not found".to_string(),
        )));
    }

    tracing::info!(
        "Successfully updated share permission for humidor {} user {}",
        humidor_id,
        user_id
    );

    Ok(reply::json(&serde_json::json!({
        "message": "Permission updated successfully"
    })))
}

/// Get list of users a humidor is shared with
/// GET /api/v1/humidors/:id/shares
pub async fn get_humidor_shares(
    humidor_id: Uuid,
    auth: AuthContext,
    pool: Pool,
) -> Result<impl Reply, Rejection> {
    tracing::debug!("User {} fetching shares for humidor {}", auth.user_id, humidor_id);

    // Verify user has access to the humidor (owner or has share)
    if !can_view_humidor(&pool, &auth.user_id, &humidor_id)
        .await
        .map_err(reject::custom)?
    {
        return Err(reject::custom(AppError::Forbidden(
            "You do not have access to this humidor".to_string(),
        )));
    }

    let client = pool.get().await.map_err(|e| {
        tracing::error!("Failed to get database connection: {}", e);
        reject::custom(AppError::DatabaseError(
            "Failed to connect to database".to_string(),
        ))
    })?;

    let rows = client
        .query(
            "SELECT hs.id, hs.humidor_id, hs.permission_level, hs.created_at, hs.updated_at,
                    u_with.id, u_with.username, u_with.email, u_with.full_name,
                    u_by.id, u_by.username, u_by.email, u_by.full_name
             FROM humidor_shares hs
             INNER JOIN users u_with ON hs.shared_with_user_id = u_with.id
             INNER JOIN users u_by ON hs.shared_by_user_id = u_by.id
             WHERE hs.humidor_id = $1
             ORDER BY hs.created_at DESC",
            &[&humidor_id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch humidor shares: {}", e);
            reject::custom(AppError::DatabaseError(
                "Failed to fetch shares".to_string(),
            ))
        })?;

    let shares: Vec<HumidorShareResponse> = rows
        .iter()
        .map(|row| {
            let permission_str: String = row.get(2);
            let permission = PermissionLevel::from_str(&permission_str)
                .unwrap_or(PermissionLevel::View);

            HumidorShareResponse {
                id: row.get(0),
                humidor_id: row.get(1),
                shared_with_user: UserInfo {
                    id: row.get(5),
                    username: row.get(6),
                    email: row.get(7),
                    full_name: row.get(8),
                },
                shared_by_user: UserInfo {
                    id: row.get(9),
                    username: row.get(10),
                    email: row.get(11),
                    full_name: row.get(12),
                },
                permission_level: permission,
                created_at: row.get(3),
                updated_at: row.get(4),
            }
        })
        .collect();

    let total = shares.len();

    Ok(reply::json(&HumidorSharesListResponse { shares, total }))
}

/// Get list of humidors shared with the current user
/// GET /api/v1/humidors/shared
pub async fn get_shared_humidors(auth: AuthContext, pool: Pool) -> Result<impl Reply, Rejection> {
    tracing::debug!("User {} fetching shared humidors", auth.user_id);

    let client = pool.get().await.map_err(|e| {
        tracing::error!("Failed to get database connection: {}", e);
        reject::custom(AppError::DatabaseError(
            "Failed to connect to database".to_string(),
        ))
    })?;

    let rows = client
        .query(
            "SELECT h.id, h.name, h.description, hs.permission_level, hs.created_at,
                    u.id, u.username, u.email, u.full_name,
                    COUNT(c.id) as cigar_count
             FROM humidor_shares hs
             INNER JOIN humidors h ON hs.humidor_id = h.id
             INNER JOIN users u ON h.user_id = u.id
             LEFT JOIN cigars c ON c.humidor_id = h.id
             WHERE hs.shared_with_user_id = $1
             GROUP BY h.id, h.name, h.description, hs.permission_level, hs.created_at,
                      u.id, u.username, u.email, u.full_name
             ORDER BY hs.created_at DESC",
            &[&auth.user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch shared humidors: {}", e);
            reject::custom(AppError::DatabaseError(
                "Failed to fetch shared humidors".to_string(),
            ))
        })?;

    let humidors: Vec<SharedHumidorInfo> = rows
        .iter()
        .map(|row| {
            let permission_str: String = row.get(3);
            let permission = PermissionLevel::from_str(&permission_str)
                .unwrap_or(PermissionLevel::View);

            SharedHumidorInfo {
                id: row.get(0),
                name: row.get(1),
                description: row.get(2),
                owner: UserInfo {
                    id: row.get(5),
                    username: row.get(6),
                    email: row.get(7),
                    full_name: row.get(8),
                },
                permission_level: permission,
                shared_at: row.get(4),
                cigar_count: row.get(9),
            }
        })
        .collect();

    let total = humidors.len();

    Ok(reply::json(&SharedHumidorsResponse { humidors, total }))
}
