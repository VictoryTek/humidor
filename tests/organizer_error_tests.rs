//! Organizer and cigar handlers must report failures with real HTTP statuses (not `200` with an
//! `{"error": ...}` body): duplicate names -> 409, missing rows -> 404.

mod common;

use common::*;
use humidor::errors::handle_rejection;
use humidor::routes::{create_cigar_routes, create_organizer_routes};
use serde_json::{Value, json};
use serial_test::serial;
use uuid::Uuid;
use warp::Filter;
use warp::http::StatusCode;

async fn call<F, R>(
    api: &F,
    token: &str,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value)
where
    F: Filter<Extract = (R,), Error = std::convert::Infallible> + Clone + 'static,
    R: warp::Reply + Send + 'static,
{
    let mut req = warp::test::request()
        .method(method)
        .path(path)
        .header("authorization", format!("Bearer {token}"));
    if let Some(b) = body {
        req = req
            .header("content-type", "application/json")
            .body(b.to_string());
    }
    let res = req.reply(api).await;
    let json = serde_json::from_slice(res.body()).unwrap_or(Value::Null);
    (res.status(), json)
}

async fn user_token(ctx: &TestContext) -> String {
    let (id, name) = create_test_user(&ctx.pool, "org", "Passw0rd!", false)
        .await
        .unwrap();
    create_test_jwt(id, &name).unwrap()
}

#[tokio::test]
#[serial]
async fn duplicate_organizer_names_are_409_for_every_organizer_type() {
    let ctx = setup_test_db().await;
    let api = create_organizer_routes(ctx.pool.clone()).recover(handle_rejection);
    let token = user_token(&ctx).await;

    let payloads = [
        ("brands", json!({"name": "Dup Brand"})),
        ("sizes", json!({"name": "Dup Size"})),
        (
            "origins",
            json!({"name": "Dup Origin", "country": "Nowhere"}),
        ),
        ("strengths", json!({"name": "Dup Strength", "level": 3})),
        ("ring-gauges", json!({"gauge": 77})),
    ];
    for (kind, payload) in payloads {
        let path = format!("/api/v1/{kind}");
        let (status, _) = call(&api, &token, "POST", &path, Some(payload.clone())).await;
        assert_eq!(status, StatusCode::OK, "first create of {kind}");

        let (status, body) = call(&api, &token, "POST", &path, Some(payload)).await;
        assert_eq!(status, StatusCode::CONFLICT, "duplicate {kind}");
        assert_eq!(body["error"], "CONFLICT");
        assert!(
            body["message"]
                .as_str()
                .unwrap()
                .ends_with("already exists")
        );
    }
}

#[tokio::test]
#[serial]
async fn missing_organizers_are_404_on_update_and_delete() {
    let ctx = setup_test_db().await;
    let api = create_organizer_routes(ctx.pool.clone()).recover(handle_rejection);
    let token = user_token(&ctx).await;
    let missing = Uuid::new_v4();

    for kind in ["brands", "sizes", "origins", "strengths", "ring-gauges"] {
        let path = format!("/api/v1/{kind}/{missing}");
        let (status, body) = call(&api, &token, "DELETE", &path, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "DELETE {kind}");
        assert_eq!(body["error"], "NOT_FOUND");

        let (status, body) = call(
            &api,
            &token,
            "PUT",
            &path,
            Some(json!({"name": "x", "gauge": 50, "level": 1})),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND, "PUT {kind}");
        assert_eq!(body["error"], "NOT_FOUND");
    }
}

#[tokio::test]
#[serial]
async fn renaming_an_organizer_to_an_existing_name_is_409() {
    let ctx = setup_test_db().await;
    let api = create_organizer_routes(ctx.pool.clone()).recover(handle_rejection);
    let token = user_token(&ctx).await;

    call(
        &api,
        &token,
        "POST",
        "/api/v1/brands",
        Some(json!({"name": "First"})),
    )
    .await;
    let (_, second) = call(
        &api,
        &token,
        "POST",
        "/api/v1/brands",
        Some(json!({"name": "Second"})),
    )
    .await;
    let id = second["id"].as_str().unwrap();

    let (status, body) = call(
        &api,
        &token,
        "PUT",
        &format!("/api/v1/brands/{id}"),
        Some(json!({"name": "First"})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "CONFLICT");
}

#[tokio::test]
#[serial]
async fn cigar_with_unknown_brand_is_400_not_200() {
    let ctx = setup_test_db().await;
    let api = create_cigar_routes(ctx.pool.clone()).recover(handle_rejection);
    let (user_id, username) = create_test_user(&ctx.pool, "cig", "Passw0rd!", false)
        .await
        .unwrap();
    let token = create_test_jwt(user_id, &username).unwrap();
    let humidor_id = create_test_humidor(&ctx.pool, user_id, "Owned")
        .await
        .unwrap();

    let (status, body) = call(
        &api,
        &token,
        "POST",
        "/api/v1/cigars",
        Some(json!({"name": "Ghost", "quantity": 1, "humidor_id": humidor_id, "brand_id": Uuid::new_v4()})),
    )
    .await;
    // Foreign-key violation on brand_id: a real 400, never a 200 with an error body.
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["message"], "Referenced record does not exist");
}

#[tokio::test]
#[serial]
async fn missing_cigar_is_404_on_get_and_delete() {
    let ctx = setup_test_db().await;
    let api = create_cigar_routes(ctx.pool.clone()).recover(handle_rejection);
    let token = user_token(&ctx).await;
    let path = format!("/api/v1/cigars/{}", Uuid::new_v4());

    for method in ["GET", "DELETE"] {
        let (status, _) = call(&api, &token, method, &path, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{method} missing cigar");
    }
}

#[tokio::test]
#[serial]
async fn scrape_failure_is_a_400_with_a_message() {
    let ctx = setup_test_db().await;
    let api = create_cigar_routes(ctx.pool.clone()).recover(handle_rejection);
    let token = user_token(&ctx).await;

    let (status, body) = call(
        &api,
        &token,
        "POST",
        "/api/v1/cigars/scrape",
        Some(json!({"url": "http://127.0.0.1/"})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        body["message"],
        "Could not scrape cigar information from that URL"
    );
}
