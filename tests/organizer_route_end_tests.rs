//! Regression for the missing `path::end()` on organizer update/delete routes (same bug class as
//! the humidor-delete fallthrough): a garbage-suffixed path must not silently match, and normal
//! update/delete requests must keep working.

mod common;

use common::*;
use humidor::errors::handle_rejection;
use humidor::routes::create_organizer_routes;
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
    let api = create_organizer_routes(ctx.pool.clone()).recover(handle_rejection);
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

async fn user_token(ctx: &TestContext) -> String {
    let (id, name) = create_test_user(&ctx.pool, "orgend", "Passw0rd!", false)
        .await
        .unwrap();
    create_test_jwt(id, &name).unwrap()
}

#[tokio::test]
#[serial]
async fn garbage_suffixed_update_and_delete_no_longer_silently_match() {
    let ctx = setup_test_db().await;
    let token = user_token(&ctx).await;

    let creations: [(&str, Value); 5] = [
        ("brands", json!({"name": "Brand X"})),
        ("sizes", json!({"name": "Size X"})),
        ("origins", json!({"name": "Origin X", "country": "Nowhere"})),
        ("strengths", json!({"name": "Strength X", "level": 3})),
        ("ring-gauges", json!({"gauge": 42})),
    ];

    for (kind, payload) in creations {
        let (status, created) = call(
            &ctx,
            &token,
            "POST",
            &format!("/api/v1/{kind}"),
            Some(payload),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "create {kind}: {created}");
        let id = created["id"].as_str().unwrap();

        // Before the fix, both of these matched update/delete despite the extra path segment.
        for (method, body) in [
            (
                "PUT",
                Some(json!({"name": "hacked", "gauge": 1, "level": 1})),
            ),
            ("DELETE", None),
        ] {
            let path = format!("/api/v1/{kind}/{id}/garbage");
            let (status, body_resp) = call(&ctx, &token, method, &path, body).await;
            assert_ne!(
                status,
                StatusCode::OK,
                "{method} {path} must not silently succeed: {body_resp}"
            );
        }

        // The record must be untouched by the rejected garbage-suffixed requests above.
        let (status, fetched) = call(&ctx, &token, "GET", &format!("/api/v1/{kind}"), None).await;
        assert_eq!(status, StatusCode::OK);
        let still_present = fetched
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == id && item["name"] != "hacked");
        assert!(
            still_present,
            "{kind} record must survive the garbage-suffixed requests: {fetched}"
        );
    }
}

#[tokio::test]
#[serial]
async fn normal_update_and_delete_still_work_for_every_organizer_type() {
    let ctx = setup_test_db().await;
    let token = user_token(&ctx).await;

    let cases: [(&str, Value, Value); 5] = [
        (
            "brands",
            json!({"name": "B1"}),
            json!({"name": "B1 renamed"}),
        ),
        (
            "sizes",
            json!({"name": "S1"}),
            json!({"name": "S1 renamed"}),
        ),
        (
            "origins",
            json!({"name": "O1", "country": "X"}),
            json!({"name": "O1 renamed"}),
        ),
        (
            "strengths",
            json!({"name": "St1", "level": 2}),
            json!({"name": "St1 renamed"}),
        ),
        ("ring-gauges", json!({"gauge": 50}), json!({"gauge": 52})),
    ];

    for (kind, create, update) in cases {
        let (_, created) = call(
            &ctx,
            &token,
            "POST",
            &format!("/api/v1/{kind}"),
            Some(create),
        )
        .await;
        let id = created["id"].as_str().unwrap().to_string();

        let (status, updated) = call(
            &ctx,
            &token,
            "PUT",
            &format!("/api/v1/{kind}/{id}"),
            Some(update),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "update {kind}: {updated}");

        let (status, _) = call(
            &ctx,
            &token,
            "DELETE",
            &format!("/api/v1/{kind}/{id}"),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "delete {kind}");

        let (_, fetched) = call(&ctx, &token, "GET", &format!("/api/v1/{kind}"), None).await;
        assert!(
            fetched
                .as_array()
                .unwrap()
                .iter()
                .all(|item| item["id"] != id),
            "{kind} record must actually be gone after delete"
        );
    }

    let _ = Uuid::new_v4(); // keep uuid import used if the id parsing above ever changes
}
