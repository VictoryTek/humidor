use super::helpers::{with_current_user, with_db};
use crate::DbPool;
use crate::handlers;
use crate::handlers::export::ExportQuery;
use warp::Filter;

/// Per-user data export (requires authentication)
pub fn create_export_routes(
    db_pool: DbPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("export"))
        .and(warp::path::end())
        .and(warp::get())
        .and(warp::query::<ExportQuery>())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool))
        .and_then(handlers::export::export_collection)
}
