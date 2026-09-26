//! Profile/password/setup-status failures must use real HTTP statuses. In particular a wrong current
//! password must be a 400 (NOT 401: the frontend logs the user out on any 401) and must not change
//! the stored password.

mod common;

use common::*;
use humidor::errors::handle_rejection;
use humidor::routes::create_user_routes;
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
    body: Value,
) -> (StatusCode, Value)
where
    F: Filter<Extract = (R,), Error = std::convert::Infallible> + Clone + 'static,
    R: warp::Reply + Send + 'static,
{
    let res = warp::test::request()
        .method(method)
        .path(path)
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(body.to_string())
        .reply(api)
        .await;
    let json = serde_json::from_slice(res.body()).unwrap_or(Value::Null);
    (res.status(), json)
}

async fn password_hash(ctx: &TestContext, user_id: Uuid) -> String {
    ctx.pool
        .get()
        .await
        .unwrap()
        .query_one("SELECT password_hash FROM users WHERE id = $1", &[&user_id])
        .await
        .unwrap()
        .get(0)
}

#[tokio::test]
#[serial]
async fn wrong_current_password_is_400_and_changes_nothing() {
    let ctx = setup_test_db().await;
    let api = create_user_routes(ctx.pool.clone()).recover(handle_rejection);
    let (user_id, username) = create_test_user(&ctx.pool, "pw", "OldPassw0rd!", false)
        .await
        .unwrap();
    let token = create_test_jwt(user_id, &username).unwrap();
    let before = password_hash(&ctx, user_id).await;

    let (status, body) = call(
        &api,
        &token,
        "PUT",
        "/api/v1/users/password",
        json!({"current_password": "definitely-wrong", "new_password": "NewPassw0rd!"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_ne!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["message"], "Current password is incorrect");
    assert_eq!(
        password_hash(&ctx, user_id).await,
        before,
        "password must be unchanged"
    );
}

#[tokio::test]
#[serial]
async fn correct_current_password_changes_it() {
    let ctx = setup_test_db().await;
    let api = create_user_routes(ctx.pool.clone()).recover(handle_rejection);
    let (user_id, username) = create_test_user(&ctx.pool, "pw", "OldPassw0rd!", false)
        .await
        .unwrap();
    let token = create_test_jwt(user_id, &username).unwrap();

    let (status, body) = call(
        &api,
        &token,
        "PUT",
        "/api/v1/users/password",
        json!({"current_password": "OldPassw0rd!", "new_password": "NewPassw0rd!"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["message"], "Password updated successfully");
    assert!(bcrypt::verify("NewPassw0rd!", &password_hash(&ctx, user_id).await).unwrap());
}

#[tokio::test]
#[serial]
async fn updating_email_to_one_already_taken_is_409() {
    let ctx = setup_test_db().await;
    let api = create_user_routes(ctx.pool.clone()).recover(handle_rejection);
    let (other_id, _) = create_test_user(&ctx.pool, "other", "Passw0rd!", false)
        .await
        .unwrap();
    let other_email: String = ctx
        .pool
        .get()
        .await
        .unwrap()
        .query_one("SELECT email FROM users WHERE id = $1", &[&other_id])
        .await
        .unwrap()
        .get(0);
    let (user_id, username) = create_test_user(&ctx.pool, "me", "Passw0rd!", false)
        .await
        .unwrap();
    let token = create_test_jwt(user_id, &username).unwrap();

    let (status, body) = call(
        &api,
        &token,
        "PUT",
        "/api/v1/users/self",
        json!({"email": other_email}),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["message"], "Username or email already exists");

    let (status, body) = call(
        &api,
        &token,
        "PUT",
        "/api/v1/users/self",
        json!({"full_name": "Renamed Person"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["full_name"], "Renamed Person");
}
