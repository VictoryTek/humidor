use crate::errors::AppError;
use crate::middleware::auth::AuthContext;
use crate::services::backup::{
    BackupInfo, backup_path, create_backup, delete_backup, list_backups, restore_backup,
    restore_backup_from_path,
};
use bytes::Buf;
use deadpool_postgres::Pool as DbPool;
use futures::StreamExt;
use serde::Serialize;
use std::path::Path;
use warp::{Rejection, Reply};

#[derive(Serialize)]
pub struct BackupsResponse {
    pub backups: Vec<BackupInfo>,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

pub async fn get_backups(_auth: AuthContext, _pool: DbPool) -> Result<impl Reply, Rejection> {
    match list_backups() {
        Ok(backups) => Ok(warp::reply::json(&BackupsResponse { backups })),
        Err(e) => {
            tracing::error!(error = %e, "Error listing backups");
            Ok(warp::reply::json(&BackupsResponse {
                backups: Vec::new(),
            }))
        }
    }
}

pub async fn create_backup_handler(
    _auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::reject()
    })?;

    match create_backup(&db).await {
        Ok(backup_name) => Ok(warp::reply::json(&MessageResponse {
            message: format!("Backup created successfully: {}", backup_name),
        })),
        Err(e) => {
            tracing::error!(error = %e, "Error creating backup");
            Ok(warp::reply::json(&MessageResponse {
                message: format!("Error creating backup: {}", e),
            }))
        }
    }
}

pub async fn download_backup(
    filename: String,
    _auth: AuthContext,
    _pool: DbPool,
) -> Result<impl Reply, Rejection> {
    let backup_path = backup_path(&filename).map_err(|_| warp::reject::not_found())?;
    if !backup_path.exists() {
        return Err(warp::reject::not_found());
    }

    // Read the file
    match tokio::fs::read(&backup_path).await {
        Ok(contents) => {
            let response = warp::http::Response::builder()
                .header("Content-Type", "application/zip")
                .header(
                    "Content-Disposition",
                    format!("attachment; filename=\"{}\"", filename),
                )
                .body(contents)
                .map_err(|e| {
                    tracing::error!(error = %e, "Failed to build HTTP response for backup download");
                    warp::reject::reject()
                })?;
            Ok(response)
        }
        Err(e) => {
            tracing::error!(
                filename = %filename,
                error = %e,
                "Error reading backup file"
            );
            Err(warp::reject::not_found())
        }
    }
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
        Err(e) => {
            tracing::error!(error = %e, "Error deleting backup");
            Ok(warp::reply::json(&MessageResponse {
                message: format!("Error deleting backup: {}", e),
            }))
        }
    }
}

pub async fn restore_backup_handler(
    filename: String,
    _auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::reject()
    })?;

    match restore_backup(&db, &filename).await {
        Ok(()) => Ok(warp::reply::json(&MessageResponse {
            message: "Backup restored successfully. Please refresh the page.".to_string(),
        })),
        Err(e) => {
            tracing::error!(error = %e, "Error restoring backup");
            Ok(warp::reply::json(&MessageResponse {
                message: format!("Error restoring backup: {}", e),
            }))
        }
    }
}

pub async fn upload_backup(
    _auth: AuthContext,
    form: warp::multipart::FormData,
    _pool: DbPool,
) -> Result<impl Reply, Rejection> {
    use futures::StreamExt;

    let backups_dir = Path::new("backups");
    std::fs::create_dir_all(backups_dir).map_err(|e| {
        tracing::error!(error = %e, "Failed to create backups directory");
        warp::reject::reject()
    })?;

    let mut parts = form;

    while let Some(Ok(mut part)) = parts.next().await {
        if part.name() == "file" {
            let filename = part.filename().unwrap_or("backup.zip").to_string();

            // Security check: bare *.zip filename only (no path components)
            let Ok(backup_path) = backup_path(&filename) else {
                return Ok(warp::reply::json(&MessageResponse {
                    message: "Invalid filename: only plain .zip filenames are allowed".to_string(),
                }));
            };

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
            tokio::fs::write(&backup_path, &buffer).await.map_err(|e| {
                tracing::error!(error = %e, "Error writing file");
                warp::reject::reject()
            })?;

            return Ok(warp::reply::json(&MessageResponse {
                message: format!("Backup {} uploaded successfully", filename),
            }));
        }
    }

    Ok(warp::reply::json(&MessageResponse {
        message: "No file provided".to_string(),
    }))
}

// Setup restore - upload and restore backup during initial setup.
// No auth, so it is only permitted while no admin exists (same rule as get_setup_status).
pub async fn setup_restore_backup(
    mut form: warp::multipart::FormData,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    let db = pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get database connection");
        warp::reject::reject()
    })?;
    let admin_count: i64 = db
        .query_one("SELECT COUNT(*) FROM users WHERE is_admin = true", &[])
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to check setup status");
            warp::reject::reject()
        })?
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
                return Ok(warp::reply::json(&MessageResponse {
                    message: "Invalid file type. Only .zip files are allowed".to_string(),
                }));
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
            tokio::fs::write(&backup_path, &buffer).await.map_err(|e| {
                tracing::error!(error = %e, "Error writing file");
                warp::reject::reject()
            })?;

            // Now restore the backup
            // Convert error to String immediately to avoid Send issues across await
            let result = restore_backup_from_path(&db, &backup_path)
                .await
                .map_err(|e| format!("Error restoring backup: {}", e));

            // Clean up the uploaded file regardless of success/failure
            let _ = tokio::fs::remove_file(&backup_path).await;

            match result {
                Ok(_) => {
                    return Ok(warp::reply::json(&MessageResponse {
                        message: "Backup restored successfully".to_string(),
                    }));
                }
                Err(error_msg) => {
                    tracing::error!(message = %error_msg, "Backup error");
                    return Ok(warp::reply::json(&MessageResponse { message: error_msg }));
                }
            }
        }
    }

    Ok(warp::reply::json(&MessageResponse {
        message: "No file provided".to_string(),
    }))
}
