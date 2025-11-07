use super::helpers::{json_body, with_current_user, with_db};
use crate::handlers;
use crate::DbPool;
use warp::Filter;

/// Create all user profile-related routes (requires authentication)
pub fn create_user_routes(
    db_pool: DbPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let get_current_user = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("users"))
        .and(warp::path("self"))
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_current_user);

    let update_current_user = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("users"))
        .and(warp::path("self"))
        .and(warp::put())
        .and(json_body())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_current_user);

    let change_password = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("users"))
        .and(warp::path("password"))
        .and(warp::put())
        .and(json_body())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::change_password);

    get_current_user.or(update_current_user).or(change_password)
}
