use super::helpers::{json_body, with_current_user, with_db, with_uuid};
use crate::DbPool;
use crate::handlers;
use warp::Filter;

/// Create all favorites and wish list routes (requires authentication)
pub fn create_favorite_routes(
    db_pool: DbPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    // Favorites routes
    let get_favorites = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("favorites"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_favorites);

    let add_favorite = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("favorites"))
        .and(warp::path::end())
        .and(warp::post())
        .and(json_body())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::add_favorite);

    let remove_favorite = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("favorites"))
        .and(with_uuid())
        .and(warp::path::end())
        .and(warp::delete())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::remove_favorite);

    let check_favorite = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("favorites"))
        .and(with_uuid())
        .and(warp::path("check"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::is_favorite);

    // Wish List routes
    let get_wish_list = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("wish_list"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_wish_list);

    let add_to_wish_list = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("wish_list"))
        .and(warp::path::end())
        .and(warp::post())
        .and(json_body())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::add_to_wish_list);

    let remove_from_wish_list = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("wish_list"))
        .and(with_uuid())
        .and(warp::path::end())
        .and(warp::delete())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::remove_from_wish_list);

    let check_wish_list = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("wish_list"))
        .and(with_uuid())
        .and(warp::path("check"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::check_wish_list);

    let update_wish_list_notes = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("wish_list"))
        .and(with_uuid())
        .and(warp::path::end())
        .and(warp::put())
        .and(json_body())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_wish_list_notes);

    // Must come before remove_favorite (more specific route)
    check_favorite
        .or(get_favorites)
        .or(add_favorite)
        .or(remove_favorite)
        .or(check_wish_list) // Must come before remove_from_wish_list (more specific route)
        .or(update_wish_list_notes) // Must come before remove_from_wish_list (both have UUID path)
        .or(get_wish_list)
        .or(add_to_wish_list)
        .or(remove_from_wish_list)
}
