use super::helpers::{json_body, with_current_user, with_db, with_uuid};
use crate::DbPool;
use crate::handlers;
use warp::Filter;

/// Smoking journal routes: logging/listing sessions on a cigar, and a user's own session history.
pub fn create_smoking_session_routes(
    db_pool: DbPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let create_session = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("cigars"))
        .and(with_uuid())
        .and(warp::path("sessions"))
        .and(warp::path::end())
        .and(warp::post())
        .and(with_current_user(db_pool.clone()))
        .and(json_body())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::smoking_sessions::create_session);

    let get_cigar_sessions = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("cigars"))
        .and(with_uuid())
        .and(warp::path("sessions"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::smoking_sessions::get_cigar_sessions);

    let get_my_sessions = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("sessions"))
        .and(warp::path::end())
        .and(warp::get())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::smoking_sessions::get_my_sessions);

    create_session.or(get_cigar_sessions).or(get_my_sessions)
}
