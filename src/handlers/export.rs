use crate::errors::AppError;
use crate::middleware::auth::AuthContext;
use crate::services::export::{collect_user_export, to_csv};
use deadpool_postgres::Pool as DbPool;
use serde::Deserialize;
use warp::{Rejection, Reply};

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>,
}

/// Export the requesting user's own collection.
/// GET /api/v1/export?format=json|csv   (default: json)
pub async fn export_collection(
    query: ExportQuery,
    auth: AuthContext,
    pool: DbPool,
) -> Result<impl Reply, Rejection> {
    let format = query.format.as_deref().unwrap_or("json");
    if format != "json" && format != "csv" {
        return Err(warp::reject::custom(AppError::BadRequest(
            "format must be 'json' or 'csv'".to_string(),
        )));
    }

    let db = pool.get().await.map_err(|e| {
        warp::reject::custom(AppError::DatabaseError(format!(
            "Failed to get database connection: {e}"
        )))
    })?;

    let export = collect_user_export(&db, auth.user_id)
        .await
        .map_err(|e| warp::reject::custom(AppError::DatabaseError(e.to_string())))?;

    let (content_type, body) = if format == "csv" {
        ("text/csv; charset=utf-8", to_csv(&export).into_bytes())
    } else {
        let json = serde_json::to_vec_pretty(&export).map_err(|e| {
            warp::reject::custom(AppError::InternalServerError(format!(
                "Failed to serialize export: {e}"
            )))
        })?;
        ("application/json", json)
    };

    let filename = format!(
        "humidor-collection-{}.{}",
        export.exported_at.format("%Y-%m-%d"),
        format
    );

    warp::http::Response::builder()
        .header("Content-Type", content_type)
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{filename}\""),
        )
        // Personal data: never store in any cache
        .header("Cache-Control", "no-store")
        .body(body)
        .map_err(|e| {
            warp::reject::custom(AppError::InternalServerError(format!(
                "Failed to build export response: {e}"
            )))
        })
}
