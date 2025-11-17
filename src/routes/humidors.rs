use super::helpers::{json_body, with_current_user, with_db, with_uuid};
use crate::DbPool;
use crate::handlers;
use warp::Filter;

/// Create all humidor-related routes (requires authentication)
pub fn create_humidor_routes(
    db_pool: DbPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let get_humidors = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("humidors"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_humidors);

    let get_humidor = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("humidors"))
        .and(with_uuid())
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_humidor);

    let create_humidor = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("humidors"))
        .and(warp::post())
        .and(json_body())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::create_humidor);

    let update_humidor = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("humidors"))
        .and(with_uuid())
        .and(warp::put())
        .and(json_body())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_humidor);

    let delete_humidor = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("humidors"))
        .and(with_uuid())
        .and(warp::delete())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::delete_humidor);

    let get_humidor_cigars = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("humidors"))
        .and(with_uuid())
        .and(warp::path("cigars"))
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_humidor_cigars);

    // Humidor sharing routes
    let get_shared_humidors = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("humidors"))
        .and(warp::path("shared"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_shared_humidors);

    let get_humidor_shares = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("humidors"))
        .and(with_uuid())
        .and(warp::path("shares"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_humidor_shares);

    let share_humidor = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("humidors"))
        .and(with_uuid())
        .and(warp::path("share"))
        .and(warp::path::end())
        .and(warp::post())
        .and(with_current_user(db_pool.clone()))
        .and(json_body())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::share_humidor);

    let revoke_share = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("humidors"))
        .and(with_uuid())
        .and(warp::path("share"))
        .and(with_uuid())
        .and(warp::path::end())
        .and(warp::delete())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::revoke_share);

    let update_share_permission = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("humidors"))
        .and(with_uuid())
        .and(warp::path("share"))
        .and(with_uuid())
        .and(warp::path::end())
        .and(warp::patch())
        .and(with_current_user(db_pool.clone()))
        .and(json_body())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_share_permission);

    // Must come before get_humidor (more specific route)
    get_humidors
        .or(get_shared_humidors) // More specific, should come early
        .or(get_humidor_cigars)
        .or(get_humidor_shares)
        .or(share_humidor)
        .or(revoke_share)
        .or(update_share_permission)
        .or(create_humidor)
        .or(update_humidor)
        .or(delete_humidor)
        .or(get_humidor) // Less specific, should be last
}
