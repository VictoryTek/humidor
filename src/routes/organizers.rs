use super::helpers::{json_body, with_current_user, with_db, with_uuid};
use crate::DbPool;
use crate::handlers;
use warp::Filter;

/// Create all organizer routes (brands, origins, sizes, strengths, ring gauges)
/// These routes now require authentication as they are user-specific
pub fn create_organizer_routes(
    db_pool: DbPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    // Brand routes
    let get_brands = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("brands"))
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_brands);

    let create_brand = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("brands"))
        .and(warp::post())
        .and(with_current_user(db_pool.clone()))
        .and(json_body())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::create_brand);

    let update_brand = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("brands"))
        .and(with_uuid())
        .and(warp::put())
        .and(with_current_user(db_pool.clone()))
        .and(json_body())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_brand);

    let delete_brand = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("brands"))
        .and(with_uuid())
        .and(warp::delete())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::delete_brand);

    // Size routes
    let get_sizes = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("sizes"))
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_sizes);

    let create_size = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("sizes"))
        .and(warp::post())
        .and(with_current_user(db_pool.clone()))
        .and(json_body())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::create_size);

    let update_size = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("sizes"))
        .and(with_uuid())
        .and(warp::put())
        .and(with_current_user(db_pool.clone()))
        .and(json_body())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_size);

    let delete_size = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("sizes"))
        .and(with_uuid())
        .and(warp::delete())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::delete_size);

    // Origin routes
    let get_origins = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("origins"))
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_origins);

    let create_origin = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("origins"))
        .and(warp::post())
        .and(with_current_user(db_pool.clone()))
        .and(json_body())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::create_origin);

    let update_origin = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("origins"))
        .and(with_uuid())
        .and(warp::put())
        .and(with_current_user(db_pool.clone()))
        .and(json_body())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_origin);

    let delete_origin = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("origins"))
        .and(with_uuid())
        .and(warp::delete())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::delete_origin);

    // Strength routes
    let get_strengths = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("strengths"))
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_strengths);

    let create_strength = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("strengths"))
        .and(warp::post())
        .and(with_current_user(db_pool.clone()))
        .and(json_body())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::create_strength);

    let update_strength = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("strengths"))
        .and(with_uuid())
        .and(warp::put())
        .and(with_current_user(db_pool.clone()))
        .and(json_body())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_strength);

    let delete_strength = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("strengths"))
        .and(with_uuid())
        .and(warp::delete())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::delete_strength);

    // Ring gauge routes
    let get_ring_gauges = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("ring-gauges"))
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_ring_gauges);

    let create_ring_gauge = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("ring-gauges"))
        .and(warp::post())
        .and(with_current_user(db_pool.clone()))
        .and(json_body())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::create_ring_gauge);

    let update_ring_gauge = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("ring-gauges"))
        .and(with_uuid())
        .and(warp::put())
        .and(with_current_user(db_pool.clone()))
        .and(json_body())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_ring_gauge);

    let delete_ring_gauge = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("ring-gauges"))
        .and(with_uuid())
        .and(warp::delete())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::delete_ring_gauge);

    get_brands
        .or(create_brand)
        .or(update_brand)
        .or(delete_brand)
        .or(get_sizes)
        .or(create_size)
        .or(update_size)
        .or(delete_size)
        .or(get_origins)
        .or(create_origin)
        .or(update_origin)
        .or(delete_origin)
        .or(get_strengths)
        .or(create_strength)
        .or(update_strength)
        .or(delete_strength)
        .or(get_ring_gauges)
        .or(create_ring_gauge)
        .or(update_ring_gauge)
        .or(delete_ring_gauge)
}
