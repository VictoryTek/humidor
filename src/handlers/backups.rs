use crate::middleware::auth::AuthContext;
use crate::services::backup::{create_backup, delete_backup, list_backups, restore_backup, BackupInfo};
use bytes::Buf;
use deadpool_postgres::Pool as DbPool;
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
            eprintln!("Error listing backups: {}", e);
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
        eprintln!("Failed to get database connection: {}", e);
        warp::reject::reject()
    })?;

    match create_backup(&db).await {
        Ok(backup_name) => {
            Ok(warp::reply::json(&MessageResponse {
                message: format!("Backup created successfully: {}", backup_name),
            }))
        }
        Err(e) => {
            eprintln!("Error creating backup: {}", e);
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
    let backups_dir = Path::new("backups");
    let backup_path = backups_dir.join(&filename);

    // Security check: ensure the path is within backups directory
    if !backup_path.starts_with(backups_dir) || !backup_path.exists() {
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
                .unwrap();
            Ok(response)
        }
        Err(e) => {
            eprintln!("Error reading backup file: {}", e);
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
            eprintln!("Error deleting backup: {}", e);
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
        eprintln!("Failed to get database connection: {}", e);
        warp::reject::reject()
    })?;

    match restore_backup(&db, &filename).await {
        Ok(()) => Ok(warp::reply::json(&MessageResponse {
            message: "Backup restored successfully. Please refresh the page.".to_string(),
        })),
        Err(e) => {
            eprintln!("Error restoring backup: {}", e);
            Ok(warp::reply::json(&MessageResponse {
                message: format!("Error restoring backup: {}", e),
            }))
        }
    }
}

pub async fn upload_backup(
    form: warp::multipart::FormData,
    _auth: AuthContext,
    _pool: DbPool,
) -> Result<impl Reply, Rejection> {
    use futures::StreamExt;

    let backups_dir = Path::new("backups");
    std::fs::create_dir_all(backups_dir).unwrap();

    let mut parts = form;
    
    while let Some(Ok(mut part)) = parts.next().await {
        if part.name() == "file" {
            let filename = part.filename().unwrap_or("backup.zip").to_string();
            
            // Security check: ensure it's a zip file
            if !filename.ends_with(".zip") {
                return Ok(warp::reply::json(&MessageResponse {
                    message: "Only .zip files are allowed".to_string(),
                }));
            }

            let backup_path = backups_dir.join(&filename);
            
            // Security check: ensure the path is within backups directory
            if !backup_path.starts_with(backups_dir) {
                return Ok(warp::reply::json(&MessageResponse {
                    message: "Invalid filename".to_string(),
                }));
            }

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
                eprintln!("Error writing file: {}", e);
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
