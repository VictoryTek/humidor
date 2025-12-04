use crate::DbPool;
use crate::middleware::AuthContext;
use bytes::Buf;
use futures::StreamExt;
use serde_json::json;
use std::convert::Infallible;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use warp::{multipart::FormData, Reply, http::StatusCode, reply};

/// Maximum file size: 5MB
const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;

/// Allowed image MIME types
const ALLOWED_TYPES: &[&str] = &[
    "image/jpeg",
    "image/jpg",
    "image/png",
    "image/gif",
    "image/webp",
];

/// Upload directory path
const UPLOAD_DIR: &str = "/app/uploads";

/// Handle image upload from multipart form data
pub async fn upload_image(
    mut form: FormData,
    _auth: AuthContext,
    _pool: DbPool,
) -> Result<impl Reply, Infallible> {
    tracing::info!("Image upload request received");
    
    // Ensure upload directory exists
    if let Err(e) = fs::create_dir_all(UPLOAD_DIR).await {
        tracing::error!(error = %e, "Failed to create upload directory");
        return Ok(reply::with_status(
            reply::json(&json!({"error": "Failed to create upload directory"})),
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    let mut file_data: Option<Vec<u8>> = None;
    let mut file_extension: Option<String> = None;

    // Process multipart form data
    while let Some(Ok(mut part)) = form.next().await {
        let name = part.name().to_string();
        tracing::info!(part_name = %name, "Processing multipart part");

        if name == "image" {
            // Get content type from the part and validate
            let content_type = part.content_type().map(|s| s.to_string());
            tracing::info!(content_type = ?content_type, "Image part content type");

            if let Some(ref ct) = content_type {
                if !ALLOWED_TYPES.contains(&ct.as_str()) {
                    tracing::warn!(content_type = %ct, "Invalid image type uploaded");
                    return Ok(reply::with_status(
                        reply::json(&json!({
                            "error": "Invalid image type. Allowed: JPEG, PNG, GIF, WebP"
                        })),
                        StatusCode::BAD_REQUEST,
                    ));
                }

                // Determine file extension from MIME type
                file_extension = Some(match ct.as_str() {
                    "image/jpeg" | "image/jpg" => "jpg",
                    "image/png" => "png",
                    "image/gif" => "gif",
                    "image/webp" => "webp",
                    _ => "jpg", // fallback
                }.to_string());
            } else {
                return Ok(reply::with_status(
                    reply::json(&json!({"error": "Missing content type"})),
                    StatusCode::BAD_REQUEST,
                ));
            }

            // Read file data with size limit
            let mut buffer = Vec::new();
            while let Some(Ok(mut chunk)) = part.data().await {
                while chunk.has_remaining() {
                    if (buffer.len() + chunk.remaining()) as u64 > MAX_FILE_SIZE {
                        tracing::warn!("File size exceeds maximum allowed");
                        return Ok(reply::with_status(
                            reply::json(&json!({"error": "File size exceeds 5MB limit"})),
                            StatusCode::BAD_REQUEST,
                        ));
                    }
                    let bytes = chunk.chunk();
                    buffer.extend_from_slice(bytes);
                    let len = bytes.len();
                    chunk.advance(len);
                }
            }

            let buffer_len = buffer.len();
            file_data = Some(buffer);
            tracing::info!(size = buffer_len, "Image data received");
        }
    }

    tracing::info!(has_data = file_data.is_some(), "Finished processing multipart form");

    // Validate we received file data
    let data = match file_data {
        Some(d) if !d.is_empty() => d,
        _ => {
            tracing::warn!("No image data received");
            return Ok(reply::with_status(
                reply::json(&json!({"error": "No image file provided"})),
                StatusCode::BAD_REQUEST,
            ));
        }
    };

    let ext = file_extension.unwrap_or_else(|| "jpg".to_string());

    // Generate unique filename
    let filename = format!("{}.{}", Uuid::new_v4(), ext);
    let file_path = PathBuf::from(UPLOAD_DIR).join(&filename);

    // Write file to disk
    match fs::File::create(&file_path).await {
        Ok(mut file) => {
            if let Err(e) = file.write_all(&data).await {
                tracing::error!(error = %e, path = ?file_path, "Failed to write image file");
                return Ok(reply::with_status(
                    reply::json(&json!({"error": "Failed to save image"})),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ));
            }
        }
        Err(e) => {
            tracing::error!(error = %e, path = ?file_path, "Failed to create image file");
            return Ok(reply::with_status(
                reply::json(&json!({"error": "Failed to create image file"})),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }

    tracing::info!(filename = %filename, size = data.len(), "Image uploaded successfully");

    // Return URL to access the uploaded image
    Ok(reply::with_status(
        reply::json(&json!({
            "url": format!("/uploads/{}", filename),
            "filename": filename,
            "size": data.len()
        })),
        StatusCode::OK,
    ))
}
