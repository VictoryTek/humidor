//! Humidor authorization predicates (ownership / share permission checks). Shared by the
//! handlers; lives in `services` so no handler module owns logic other handlers depend on.

use crate::errors::AppError;
use crate::models::PermissionLevel;
use deadpool_postgres::Pool;
use std::str::FromStr;
use uuid::Uuid;

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
        let permission =
            PermissionLevel::from_str(&permission_str).map_err(AppError::ValidationError)?;
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
    Ok(permission.is_some_and(|p| p.can_view()))
}

/// Helper function to check if user can edit a humidor (add/update cigars)
pub async fn can_edit_humidor(
    pool: &Pool,
    user_id: &Uuid,
    humidor_id: &Uuid,
) -> Result<bool, AppError> {
    let permission = get_user_permission_level(pool, user_id, humidor_id).await?;
    Ok(permission.is_some_and(|p| p.can_edit()))
}

/// Helper function to check if user can manage a humidor (delete cigars, manage shares)
pub async fn can_manage_humidor(
    pool: &Pool,
    user_id: &Uuid,
    humidor_id: &Uuid,
) -> Result<bool, AppError> {
    let permission = get_user_permission_level(pool, user_id, humidor_id).await?;
    Ok(permission.is_some_and(|p| p.can_manage()))
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
