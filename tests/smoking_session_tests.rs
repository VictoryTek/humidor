//! Smoking journal (MASTER_PLAN #13): logging a session decrements quantity by exactly 1 and flips
//! is_active at 0; a cigar with no quantity left cannot be smoked (and no orphan session is
//! created); permission checks match other cigar mutations; a user's own session history is
//! correctly scoped and paginated.

mod common;

use common::*;
use humidor::errors::handle_rejection;
use humidor::routes::create_smoking_session_routes;
use serde_json::{Value, json};
use serial_test::serial;
use uuid::Uuid;
use warp::Filter;
use warp::http::StatusCode;

async fn call(
    ctx: &TestContext,
    token: &str,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let api = create_smoking_session_routes(ctx.pool.clone()).recover(handle_rejection);
    let mut req = warp::test::request()
        .method(method)
        .path(path)
        .header("authorization", format!("Bearer {token}"));
    if let Some(b) = body {
        req = req
            .header("content-type", "application/json")
            .body(b.to_string());
    }
    let res = req.reply(&api).await;
    let json = serde_json::from_slice(res.body()).unwrap_or(Value::Null);
    (res.status(), json)
}

async fn quantity_and_active(ctx: &TestContext, cigar_id: Uuid) -> (i32, bool) {
    let row = ctx
        .pool
        .get()
        .await
        .unwrap()
        .query_one(
            "SELECT quantity, is_active FROM cigars WHERE id = $1",
            &[&cigar_id],
        )
        .await
        .unwrap();
    (row.get(0), row.get(1))
}

async fn owner_with_cigar(ctx: &TestContext, quantity: i32) -> (Uuid, String, Uuid) {
    let (user_id, username) = create_test_user(&ctx.pool, "smoker", "Passw0rd!", false)
        .await
        .unwrap();
    let humidor_id = create_test_humidor(&ctx.pool, user_id, "Humidor")
        .await
        .unwrap();
    let cigar_id = create_test_cigar(&ctx.pool, "Cohiba", quantity, Some(humidor_id))
        .await
        .unwrap();
    (user_id, username, cigar_id)
}

#[tokio::test]
#[serial]
async fn logging_a_session_decrements_quantity_by_one() {
    let ctx = setup_test_db().await;
    let (user_id, username, cigar_id) = owner_with_cigar(&ctx, 3).await;
    let token = create_test_jwt(user_id, &username).unwrap();

    let (status, body) = call(
        &ctx,
        &token,
        "POST",
        &format!("/api/v1/cigars/{cigar_id}/sessions"),
        Some(json!({"rating": 5, "notes": "Excellent"})),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["rating"], 5);
    assert_eq!(body["notes"], "Excellent");
    assert_eq!(body["cigar_quantity"], 2);
    assert_eq!(body["cigar_is_active"], true);

    let (quantity, is_active) = quantity_and_active(&ctx, cigar_id).await;
    assert_eq!(quantity, 2);
    assert!(is_active);
}

#[tokio::test]
#[serial]
async fn logging_the_last_one_flips_is_active_to_false() {
    let ctx = setup_test_db().await;
    let (user_id, username, cigar_id) = owner_with_cigar(&ctx, 1).await;
    let token = create_test_jwt(user_id, &username).unwrap();

    let (status, body) = call(
        &ctx,
        &token,
        "POST",
        &format!("/api/v1/cigars/{cigar_id}/sessions"),
        Some(json!({})),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["cigar_quantity"], 0);
    assert_eq!(body["cigar_is_active"], false);

    let (quantity, is_active) = quantity_and_active(&ctx, cigar_id).await;
    assert_eq!(quantity, 0);
    assert!(!is_active);
}

#[tokio::test]
#[serial]
async fn cannot_smoke_a_cigar_with_no_quantity_left_and_no_orphan_session_is_created() {
    let ctx = setup_test_db().await;
    let (user_id, username, cigar_id) = owner_with_cigar(&ctx, 0).await;
    let token = create_test_jwt(user_id, &username).unwrap();

    let (status, body) = call(
        &ctx,
        &token,
        "POST",
        &format!("/api/v1/cigars/{cigar_id}/sessions"),
        Some(json!({})),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");

    // Atomicity: no session row must have been left behind by the rejected attempt.
    let session_count: i64 = ctx
        .pool
        .get()
        .await
        .unwrap()
        .query_one(
            "SELECT COUNT(*) FROM smoking_sessions WHERE cigar_id = $1",
            &[&cigar_id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(session_count, 0);

    let (quantity, _) = quantity_and_active(&ctx, cigar_id).await;
    assert_eq!(quantity, 0, "quantity must not go negative");
}

#[tokio::test]
#[serial]
async fn rating_out_of_range_is_rejected_before_touching_the_database() {
    let ctx = setup_test_db().await;
    let (user_id, username, cigar_id) = owner_with_cigar(&ctx, 5).await;
    let token = create_test_jwt(user_id, &username).unwrap();

    let (status, _) = call(
        &ctx,
        &token,
        "POST",
        &format!("/api/v1/cigars/{cigar_id}/sessions"),
        Some(json!({"rating": 7})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Quantity must be untouched: validation happens before the decrement.
    let (quantity, _) = quantity_and_active(&ctx, cigar_id).await;
    assert_eq!(quantity, 5);
}

#[tokio::test]
#[serial]
async fn view_only_share_can_read_but_not_log_sessions() {
    let ctx = setup_test_db().await;
    let (owner_id, _, cigar_id) = owner_with_cigar(&ctx, 5).await;
    let (viewer_id, viewer_name) = create_test_user(&ctx.pool, "viewer", "Passw0rd!", false)
        .await
        .unwrap();
    let humidor_id: Uuid = ctx
        .pool
        .get()
        .await
        .unwrap()
        .query_one("SELECT humidor_id FROM cigars WHERE id = $1", &[&cigar_id])
        .await
        .unwrap()
        .get(0);
    ctx.pool
        .get()
        .await
        .unwrap()
        .execute(
            "INSERT INTO humidor_shares (id, humidor_id, shared_with_user_id, shared_by_user_id, permission_level) \
             VALUES ($1, $2, $3, $4, 'view')",
            &[&Uuid::new_v4(), &humidor_id, &viewer_id, &owner_id],
        )
        .await
        .unwrap();
    let viewer_token = create_test_jwt(viewer_id, &viewer_name).unwrap();

    let (status, _) = call(
        &ctx,
        &viewer_token,
        "POST",
        &format!("/api/v1/cigars/{cigar_id}/sessions"),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, sessions) = call(
        &ctx,
        &viewer_token,
        "GET",
        &format!("/api/v1/cigars/{cigar_id}/sessions"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(sessions.as_array().unwrap().len(), 0);

    let (quantity, _) = quantity_and_active(&ctx, cigar_id).await;
    assert_eq!(
        quantity, 5,
        "a forbidden request must not decrement anything"
    );
}

#[tokio::test]
#[serial]
async fn a_user_with_no_access_gets_not_found_or_forbidden_not_someone_elses_data() {
    let ctx = setup_test_db().await;
    let (_, _, cigar_id) = owner_with_cigar(&ctx, 5).await;
    let (stranger_id, stranger_name) = create_test_user(&ctx.pool, "stranger", "Passw0rd!", false)
        .await
        .unwrap();
    let token = create_test_jwt(stranger_id, &stranger_name).unwrap();

    let (status, _) = call(
        &ctx,
        &token,
        "GET",
        &format!("/api/v1/cigars/{cigar_id}/sessions"),
        None,
    )
    .await;
    assert!(
        status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND,
        "{status}"
    );
}

#[tokio::test]
#[serial]
async fn cigar_sessions_are_listed_newest_first() {
    let ctx = setup_test_db().await;
    let (user_id, username, cigar_id) = owner_with_cigar(&ctx, 10).await;
    let token = create_test_jwt(user_id, &username).unwrap();

    for note in ["first", "second", "third"] {
        call(
            &ctx,
            &token,
            "POST",
            &format!("/api/v1/cigars/{cigar_id}/sessions"),
            Some(json!({"notes": note})),
        )
        .await;
    }

    let (status, sessions) = call(
        &ctx,
        &token,
        "GET",
        &format!("/api/v1/cigars/{cigar_id}/sessions"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let notes: Vec<&str> = sessions
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["notes"].as_str().unwrap())
        .collect();
    assert_eq!(notes, vec!["third", "second", "first"]);
}

#[tokio::test]
#[serial]
async fn my_sessions_is_scoped_to_the_requesting_user_and_paginated() {
    let ctx = setup_test_db().await;
    let (user_a_id, user_a_name, cigar_a) = owner_with_cigar(&ctx, 10).await;
    let (user_b_id, user_b_name, cigar_b) = owner_with_cigar(&ctx, 10).await;
    let token_a = create_test_jwt(user_a_id, &user_a_name).unwrap();
    let token_b = create_test_jwt(user_b_id, &user_b_name).unwrap();

    for _ in 0..3 {
        call(
            &ctx,
            &token_a,
            "POST",
            &format!("/api/v1/cigars/{cigar_a}/sessions"),
            Some(json!({})),
        )
        .await;
    }
    call(
        &ctx,
        &token_b,
        "POST",
        &format!("/api/v1/cigars/{cigar_b}/sessions"),
        Some(json!({})),
    )
    .await;

    let (status, page) = call(
        &ctx,
        &token_a,
        "GET",
        "/api/v1/sessions?limit=2&offset=0",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(page["total"], 3);
    assert_eq!(page["sessions"].as_array().unwrap().len(), 2);

    let (_, page2) = call(
        &ctx,
        &token_a,
        "GET",
        "/api/v1/sessions?limit=2&offset=2",
        None,
    )
    .await;
    assert_eq!(page2["sessions"].as_array().unwrap().len(), 1);

    let (_, page_b) = call(&ctx, &token_b, "GET", "/api/v1/sessions", None).await;
    assert_eq!(page_b["total"], 1);
    let cigar_names: Vec<&str> = page_b["sessions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["cigar_name"].as_str().unwrap())
        .collect();
    assert!(cigar_names.iter().all(|n| *n == "Cohiba"));
}
