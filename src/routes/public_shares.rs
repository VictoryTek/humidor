use super::helpers::{with_db, with_uuid};
use crate::DbPool;
use crate::handlers;
use warp::Filter;

/// Create public share routes (NO AUTHENTICATION REQUIRED)
/// These routes allow anonymous access to shared humidors via token
pub fn create_public_share_routes(
    db_pool: DbPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    // GET /api/v1/shared/humidors/:token
    // Public access to humidor data via share token
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("shared"))
        .and(warp::path("humidors"))
        .and(with_uuid()) // This is the token UUID
        .and(warp::path::end())
        .and(warp::get())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_public_humidor)
}
