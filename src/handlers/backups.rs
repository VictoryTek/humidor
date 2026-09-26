use crate::errors::AppError;
use crate::middleware::auth::AuthContext;
use crate::services::backup::{
    BackupInfo, BackupInputError, backup_path, create_backup, delete_backup, list_backups,
    restore_backup, restore_backup_from_path,
};
use bytes::Buf;
use deadpool_postgres::Pool as DbPool;
use futures::StreamExt;
use serde::Serialize;
use warp::{Rejection, Reply};

#[derive(Serialize)]
pub struct BackupsResponse {
    pub backups: Vec<BackupInfo>,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

/// Classify a backup service error. Caller-caused failures become 4xx; everything else is
/// an internal error whose detail is logged but never sent to the client.
fn backup_app_error(context: &str, e: Box<dyn std::error::Error>) -> AppError {
    match e.downcast_ref::<BackupInputError>() {
        Some(BackupInputError::NotFound) => AppError::NotFound("Backup".to_string()),
        Some(BackupInputError::InvalidFilename) => {
            AppError::BadRequest("Invalid backup filename".to_string())
        }
        None if e.is::<zip::result::ZipError>() || e.is::<serde_json::Error>() => {
            AppError::BadRequest("Invalid or corrupt backup file".to_string())
        }
        None => AppError::InternalServerError(format!("{context}: {e}")),
    }
}

fn internal(context: &str, e: impl std::fmt::Display) -> Rejection {
    warp::reject::custom(AppError::InternalServerError(format!("{context}: {e}")))
}

async fn get_db(pool: &DbPool) -> Result<deadpool_postgres::Client, Rejection> {
    pool.get().await.map_err(|e| {
        warp::reject::custom(AppError::DatabaseError(format!(
            "Failed to get database connection: {e}"
        )))
    })
}

pub async fn get_backups(_auth: AuthContext, _pool: DbPool) -> Result<impl Reply, Rejection> {
    let backups = list_backups()
        .map_err(|e| warp::reject::custom(backup_app_error("Error listing backups", e)))?;
    Ok(warp::reply::json(&BackupsResponse { backups }))
}

pub async fn create_backup_handler(
    _auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    let db = get_db(&pool).await?;

    match create_backup(&db).await {
        Ok(backup_name) => Ok(warp::reply::json(&MessageResponse {
            message: format!("Backup created successfully: {}", backup_name),
        })),
        Err(e) => Err(warp::reject::custom(backup_app_error(
            "Error creating backup",
            e,
        ))),
    }
}

pub async fn download_backup(
    filename: String,
    _auth: AuthContext,
    _pool: DbPool,
) -> Result<impl Reply, Rejection> {
    let backup_path = backup_path(&filename)
        .map_err(|e| warp::reject::custom(backup_app_error("Download", e)))?;
    if !backup_path.exists() {
        return Err(warp::reject::custom(AppError::NotFound(
            "Backup".to_string(),
        )));
    }

    let contents = tokio::fs::read(&backup_path)
        .await
        .map_err(|e| internal("Error reading backup file", e))?;

    warp::http::Response::builder()
        .header("Content-Type", "application/zip")
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(contents)
        .map_err(|e| internal("Failed to build backup download response", e))
}

pub async fn delete_backup_handler(
    filename: String,
    _auth: AuthContext,
    _pool: DbPool,
) -> Result<impl Reply, Rejection> {
    match delete_backup(&filename) {
        Ok(()) => Ok(warp::reply::json(&MessageResponse {
            message: format!("Backup {} deleted successfully", filename),
        })),
        Err(e) => Err(warp::reject::custom(backup_app_error(
            "Error deleting backup",
            e,
        ))),
    }
}

pub async fn restore_backup_handler(
    filename: String,
    _auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    let db = get_db(&pool).await?;

    match restore_backup(&db, &filename).await {
        Ok(()) => Ok(warp::reply::json(&MessageResponse {
            message: "Backup restored successfully. Please refresh the page.".to_string(),
        })),
        Err(e) => Err(warp::reject::custom(backup_app_error(
            "Error restoring backup",
            e,
        ))),
    }
}

pub async fn upload_backup(
    _auth: AuthContext,
    form: warp::multipart::FormData,
    _pool: DbPool,
) -> Result<impl Reply, Rejection> {
    let backups_dir = std::path::Path::new("backups");
    std::fs::create_dir_all(backups_dir)
        .map_err(|e| internal("Failed to create backups directory", e))?;

    let mut parts = form;

    while let Some(Ok(mut part)) = parts.next().await {
        if part.name() == "file" {
            let filename = part.filename().unwrap_or("backup.zip").to_string();

            // Security check: bare *.zip filename only (no path components)
            let backup_path = backup_path(&filename).map_err(|_| {
                warp::reject::custom(AppError::BadRequest(
                    "Invalid filename: only plain .zip filenames are allowed".to_string(),
                ))
            })?;

            // Collect all data into a buffer
            let mut buffer = Vec::new();
            while let Some(Ok(mut chunk)) = part.data().await {
                // Read all bytes from the Buf
                while chunk.has_remaining() {
                    let bytes = chunk.chunk();
                    buffer.extend_from_slice(bytes);
                    let len = bytes.len();
                    chunk.advance(len);
                }
            }

            // Write to file
            tokio::fs::write(&backup_path, &buffer)
                .await
                .map_err(|e| internal("Error writing uploaded backup", e))?;

            return Ok(warp::reply::json(&MessageResponse {
                message: format!("Backup {} uploaded successfully", filename),
            }));
        }
    }

    Err(warp::reject::custom(AppError::BadRequest(
        "No file provided".to_string(),
    )))
}

// Setup restore - upload and restore backup during initial setup.
// No auth, so it is only permitted while no admin exists (same rule as get_setup_status).
pub async fn setup_restore_backup(
    mut form: warp::multipart::FormData,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    let db = get_db(&pool).await?;
    let admin_count: i64 = db
        .query_one("SELECT COUNT(*) FROM users WHERE is_admin = true", &[])
        .await
        .map_err(|e| internal("Failed to check setup status", e))?
        .get(0);
    if admin_count > 0 {
        return Err(warp::reject::custom(AppError::Forbidden(
            "Setup already completed".to_string(),
        )));
    }

    // Process multipart form data
    while let Some(Ok(mut part)) = form.next().await {
        if part.name() == "file" {
            let filename = part.filename().unwrap_or("backup.zip");

            if !filename.to_ascii_lowercase().ends_with(".zip") {
                return Err(warp::reject::custom(AppError::BadRequest(
                    "Invalid file type. Only .zip files are allowed".to_string(),
                )));
            }

            // Never use the client-supplied filename on disk: server-generated name only.
            let backup_path =
                std::env::temp_dir().join(format!("setup_restore_{}.zip", uuid::Uuid::new_v4()));

            // Collect all data into a buffer
            let mut buffer = Vec::new();
            while let Some(Ok(mut chunk)) = part.data().await {
                while chunk.has_remaining() {
                    let bytes = chunk.chunk();
                    buffer.extend_from_slice(bytes);
                    let len = bytes.len();
                    chunk.advance(len);
                }
            }

            // Write to file
            tokio::fs::write(&backup_path, &buffer)
                .await
                .map_err(|e| internal("Error writing uploaded backup", e))?;

            // Now restore the backup. Convert the error to an AppError immediately so no
            // non-Send error value is held across the await below.
            let result = restore_backup_from_path(&db, &backup_path)
                .await
                .map_err(|e| backup_app_error("Error restoring backup", e));

            // Clean up the uploaded file regardless of success/failure
            let _ = tokio::fs::remove_file(&backup_path).await;

            return match result {
                Ok(()) => Ok(warp::reply::json(&MessageResponse {
                    message: "Backup restored successfully".to_string(),
                })),
                Err(app_err) => Err(warp::reject::custom(app_err)),
            };
        }
    }

    Err(warp::reject::custom(AppError::BadRequest(
        "No file provided".to_string(),
    )))
}
