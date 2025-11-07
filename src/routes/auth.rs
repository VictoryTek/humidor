use super::helpers::{json_body, with_client_ip, with_db, with_rate_limiter};
use crate::handlers;
use crate::middleware::RateLimiter;
use crate::DbPool;
use warp::Filter;

/// Create all authentication-related routes
pub fn create_auth_routes(
    db_pool: DbPool,
    rate_limiter: RateLimiter,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let get_setup_status = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("setup"))
        .and(warp::path("status"))
        .and(warp::get())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_setup_status);

    let create_setup_user = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("setup"))
        .and(warp::path("user"))
        .and(warp::post())
        .and(json_body())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::create_setup_user);

    let login_user = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("auth"))
        .and(warp::path("login"))
        .and(warp::post())
        .and(json_body())
        .and(with_db(db_pool.clone()))
        .and(with_rate_limiter(rate_limiter))
        .and(with_client_ip())
        .and_then(handlers::login_user);

    let forgot_password = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("auth"))
        .and(warp::path("forgot-password"))
        .and(warp::post())
        .and(json_body())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::forgot_password);

    let reset_password = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("auth"))
        .and(warp::path("reset-password"))
        .and(warp::post())
        .and(json_body())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::reset_password);

    let email_config_status = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("auth"))
        .and(warp::path("email-config"))
        .and(warp::get())
        .and_then(handlers::check_email_config);

    get_setup_status
        .or(create_setup_user)
        .or(login_user)
        .or(forgot_password)
        .or(reset_password)
        .or(email_config_status)
}
