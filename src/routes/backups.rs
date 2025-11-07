use super::helpers::{with_current_user, with_db};
use crate::handlers;
use crate::DbPool;
use warp::Filter;

/// Create all backup/restore-related routes (requires authentication except setup_restore)
pub fn create_backup_routes(
    db_pool: DbPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let get_backups = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("backups"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::backups::get_backups);

    let create_backup = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("backups"))
        .and(warp::path::end())
        .and(warp::post())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::backups::create_backup_handler);

    let download_backup = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("backups"))
        .and(warp::path::param())
        .and(warp::path("download"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::backups::download_backup);

    let delete_backup = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("backups"))
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::delete())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::backups::delete_backup_handler);

    let restore_backup = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("backups"))
        .and(warp::path::param())
        .and(warp::path("restore"))
        .and(warp::path::end())
        .and(warp::post())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::backups::restore_backup_handler);

    let upload_backup = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("backups"))
        .and(warp::path("upload"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::multipart::form().max_length(100_000_000)) // 100MB max
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::backups::upload_backup);

    // Setup restore (no authentication required - used during initial setup)
    let setup_restore = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("setup"))
        .and(warp::path("restore"))
        .and(warp::post())
        .and(warp::multipart::form().max_length(100_000_000)) // 100MB max
        .and(with_db(db_pool.clone()))
        .and_then(handlers::backups::setup_restore_backup);

    get_backups
        .or(create_backup)
        .or(download_backup)
        .or(delete_backup)
        .or(restore_backup)
        .or(upload_backup)
        .or(setup_restore)
}
