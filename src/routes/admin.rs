use super::helpers::{json_body, with_db};
use crate::handlers::admin;
use crate::middleware::auth::with_admin;
use crate::DbPool;
use uuid::Uuid;
use warp::Filter;

/// Create all admin-only routes
/// All routes require admin authentication via with_admin() middleware
pub fn create_admin_routes(
    db_pool: DbPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    // GET /api/v1/admin/users?page=1&per_page=20
    let list_users = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("admin"))
        .and(warp::path("users"))
        .and(warp::get())
        .and(warp::path::end())
        .and(with_admin(db_pool.clone()))
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(with_db(db_pool.clone()))
        .and_then(
            |auth, query: std::collections::HashMap<String, String>, pool| async move {
                let page = query
                    .get("page")
                    .and_then(|p| p.parse::<i32>().ok());
                let per_page = query
                    .get("per_page")
                    .and_then(|pp| pp.parse::<i32>().ok());

                admin::list_users(auth, page, per_page, pool).await
            },
        );

    // GET /api/v1/admin/users/:id
    let get_user = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("admin"))
        .and(warp::path("users"))
        .and(warp::path::param::<Uuid>())
        .and(warp::get())
        .and(warp::path::end())
        .and(with_admin(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(admin::get_user);

    // POST /api/v1/admin/users
    let create_user = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("admin"))
        .and(warp::path("users"))
        .and(warp::post())
        .and(warp::path::end())
        .and(json_body())
        .and(with_admin(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(admin::create_user);

    // PUT /api/v1/admin/users/:id
    let update_user = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("admin"))
        .and(warp::path("users"))
        .and(warp::path::param::<Uuid>())
        .and(warp::put())
        .and(warp::path::end())
        .and(json_body())
        .and(with_admin(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(admin::update_user);

    // DELETE /api/v1/admin/users/:id
    let delete_user = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("admin"))
        .and(warp::path("users"))
        .and(warp::path::param::<Uuid>())
        .and(warp::delete())
        .and(warp::path::end())
        .and(with_admin(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(admin::delete_user);

    // PATCH /api/v1/admin/users/:id/password
    let change_password = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("admin"))
        .and(warp::path("users"))
        .and(warp::path::param::<Uuid>())
        .and(warp::path("password"))
        .and(warp::patch())
        .and(warp::path::end())
        .and(json_body())
        .and(with_admin(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(admin::change_user_password);

    // PATCH /api/v1/admin/users/:id/active
    let toggle_active = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("admin"))
        .and(warp::path("users"))
        .and(warp::path::param::<Uuid>())
        .and(warp::path("active"))
        .and(warp::patch())
        .and(warp::path::end())
        .and(json_body())
        .and(with_admin(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(admin::toggle_active);

    list_users
        .or(create_user)
        .or(get_user)
        .or(update_user)
        .or(delete_user)
        .or(toggle_active)
        .or(change_password)
        .boxed()
}
