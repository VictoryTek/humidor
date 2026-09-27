//! Share/revoke email notifications (MASTER_PLAN #15) are fire-and-forget: they must never delay,
//! change the status code of, or fail the share/revoke request itself, whether or not SMTP is
//! configured. The test environment has no SMTP_* variables set, which exercises exactly the
//! "not configured" branch that must be silent.

mod common;

use common::*;
use humidor::errors::handle_rejection;
use humidor::routes::create_humidor_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;
use warp::Filter;
use warp::http::StatusCode;

async fn owner_and_target(ctx: &TestContext) -> (Uuid, String, Uuid, Uuid) {
    let (owner_id, owner_name) = create_test_user(&ctx.pool, "owner", "Passw0rd!", false)
        .await
        .unwrap();
    let (target_id, _) = create_test_user(&ctx.pool, "target", "Passw0rd!", false)
        .await
        .unwrap();
    let humidor_id = create_test_humidor(&ctx.pool, owner_id, "Shared Humidor")
        .await
        .unwrap();
    (owner_id, owner_name, target_id, humidor_id)
}

#[tokio::test]
#[serial]
async fn sharing_a_humidor_succeeds_even_though_smtp_is_not_configured() {
    let ctx = setup_test_db().await;
    let api = create_humidor_routes(ctx.pool.clone()).recover(handle_rejection);
    let (owner_id, owner_name, target_id, humidor_id) = owner_and_target(&ctx).await;
    let token = create_test_jwt(owner_id, &owner_name).unwrap();

    let res = warp::test::request()
        .method("POST")
        .path(&format!("/api/v1/humidors/{humidor_id}/share"))
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(json!({"user_id": target_id, "permission_level": "view"}).to_string())
        .reply(&api)
        .await;

    assert_eq!(res.status(), StatusCode::CREATED, "{:?}", res.body());

    // Give the fire-and-forget tokio::spawn a moment to run and settle (it must not panic).
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // The share itself must exist regardless of whether the (unconfigured) notification "sent".
    let row = ctx
        .pool
        .get()
        .await
        .unwrap()
        .query_opt(
            "SELECT permission_level FROM humidor_shares WHERE humidor_id = $1 AND shared_with_user_id = $2",
            &[&humidor_id, &target_id],
        )
        .await
        .unwrap();
    assert_eq!(row.unwrap().get::<_, String>(0), "view");

    cleanup_db(&ctx.pool).await.unwrap();
}

#[tokio::test]
#[serial]
async fn revoking_a_share_succeeds_even_though_smtp_is_not_configured() {
    let ctx = setup_test_db().await;
    let api = create_humidor_routes(ctx.pool.clone()).recover(handle_rejection);
    let (owner_id, owner_name, target_id, humidor_id) = owner_and_target(&ctx).await;
    let token = create_test_jwt(owner_id, &owner_name).unwrap();

    ctx.pool
        .get()
        .await
        .unwrap()
        .execute(
            "INSERT INTO humidor_shares (id, humidor_id, shared_with_user_id, shared_by_user_id, permission_level) \
             VALUES ($1, $2, $3, $4, 'view')",
            &[&Uuid::new_v4(), &humidor_id, &target_id, &owner_id],
        )
        .await
        .unwrap();

    let res = warp::test::request()
        .method("DELETE")
        .path(&format!("/api/v1/humidors/{humidor_id}/share/{target_id}"))
        .header("authorization", format!("Bearer {token}"))
        .reply(&api)
        .await;

    assert_eq!(res.status(), StatusCode::OK, "{:?}", res.body());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let row = ctx
        .pool
        .get()
        .await
        .unwrap()
        .query_opt(
            "SELECT 1 FROM humidor_shares WHERE humidor_id = $1 AND shared_with_user_id = $2",
            &[&humidor_id, &target_id],
        )
        .await
        .unwrap();
    assert!(row.is_none(), "share must actually be revoked");

    cleanup_db(&ctx.pool).await.unwrap();
}

#[tokio::test]
#[serial]
async fn sharing_with_a_nonexistent_user_is_still_rejected() {
    // Regression guard: the target-user lookup was changed (SELECT id -> SELECT email) to fetch
    // the notification address; a missing/inactive user must still be a 404, not treated as found.
    let ctx = setup_test_db().await;
    let api = create_humidor_routes(ctx.pool.clone()).recover(handle_rejection);
    let (owner_id, owner_name) = create_test_user(&ctx.pool, "owner2", "Passw0rd!", false)
        .await
        .unwrap();
    let humidor_id = create_test_humidor(&ctx.pool, owner_id, "H").await.unwrap();
    let token = create_test_jwt(owner_id, &owner_name).unwrap();

    let res = warp::test::request()
        .method("POST")
        .path(&format!("/api/v1/humidors/{humidor_id}/share"))
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(json!({"user_id": Uuid::new_v4(), "permission_level": "view"}).to_string())
        .reply(&api)
        .await;

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    cleanup_db(&ctx.pool).await.unwrap();
}

#[tokio::test]
#[serial]
async fn revoking_a_nonexistent_share_is_still_a_404() {
    // Regression guard: revoke now looks up the notification target before deleting; a share that
    // never existed must still 404, not silently succeed or 500.
    let ctx = setup_test_db().await;
    let api = create_humidor_routes(ctx.pool.clone()).recover(handle_rejection);
    let (owner_id, owner_name, target_id, humidor_id) = owner_and_target(&ctx).await;
    let token = create_test_jwt(owner_id, &owner_name).unwrap();

    let res = warp::test::request()
        .method("DELETE")
        .path(&format!("/api/v1/humidors/{humidor_id}/share/{target_id}"))
        .header("authorization", format!("Bearer {token}"))
        .reply(&api)
        .await;

    // BUG (found while writing this test, not introduced by it - see chat): delete_humidor's route
    // is missing warp::path::end(), so it matches ANY DELETE under /humidors/{id}/**. Because warp's
    // `.or()` falls through to the next alternative whenever a handler's own business logic returns
    // an Err (not only on a routing mismatch), revoke_share correctly rejecting with 404 here causes
    // the request to fall through and hit delete_humidor instead, which DELETES THE ENTIRE HUMIDOR
    // (the DELETE FROM humidors WHERE id=$1 AND user_id=$2 query matches, since the caller owns it).
    // This assertion documents the intended behaviour; it will fail until that route is fixed.
    let humidor_still_exists: bool = ctx
        .pool
        .get()
        .await
        .unwrap()
        .query_opt("SELECT 1 FROM humidors WHERE id = $1", &[&humidor_id])
        .await
        .unwrap()
        .is_some();
    assert!(
        humidor_still_exists,
        "revoking a nonexistent share must not delete the humidor itself"
    );
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    cleanup_db(&ctx.pool).await.unwrap();
}
