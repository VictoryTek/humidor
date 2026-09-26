//! Database failures inside handlers must surface as 500 (via `AppError::DatabaseError`), not the
//! 404 that a bare `warp::reject::reject()` produces. A pool pointed at an empty database (no
//! tables) makes every query fail while connections still succeed.

mod common;

use common::*;
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use humidor::errors::{AppError, handle_rejection};
use humidor::handlers;
use humidor::middleware::AuthContext;
use humidor::models::ForgotPasswordRequest;
use serial_test::serial;
use tokio_postgres::NoTls;
use uuid::Uuid;
use warp::Reply;
use warp::http::StatusCode;

const EMPTY_DB: &str = "humidor_empty_test";

async fn pool_without_tables() -> Pool {
    let ctx = setup_test_db().await;
    let base = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://humidor_user:humidor_pass@localhost:5432/humidor_db".to_string()
    });
    // Ignore the error if the database already exists.
    let _ = ctx
        .pool
        .get()
        .await
        .unwrap()
        .execute(&format!("CREATE DATABASE {EMPTY_DB}"), &[])
        .await;

    let (prefix, _) = base.rsplit_once('/').unwrap();
    let mut config = Config::new();
    config.url = Some(format!("{prefix}/{EMPTY_DB}"));
    config.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap()
}

/// Assert the rejection is a DatabaseError and that it renders as a generic 500.
async fn assert_database_error_500(rejection: warp::Rejection, what: &str) {
    assert!(
        matches!(
            rejection.find::<AppError>(),
            Some(AppError::DatabaseError(_))
        ),
        "{what}: expected AppError::DatabaseError, got {rejection:?}"
    );
    let response = handle_rejection(rejection).await.unwrap().into_response();
    assert_eq!(
        response.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "{what}"
    );
    let body = warp::hyper::body::to_bytes(response.into_body())
        .await
        .unwrap();
    let text = String::from_utf8_lossy(&body).to_lowercase();
    assert!(
        !text.contains("relation") && !text.contains("does not exist"),
        "{what}: DB error text leaked: {text}"
    );
}

fn auth() -> AuthContext {
    AuthContext::new(Uuid::new_v4(), "nobody".to_string())
}

#[tokio::test]
#[serial]
async fn favorites_db_failures_are_500() {
    let pool = pool_without_tables().await;

    match handlers::favorites::get_favorites(auth(), pool.clone()).await {
        Ok(_) => panic!("get_favorites should fail"),
        Err(e) => assert_database_error_500(e, "get_favorites").await,
    }
    match handlers::favorites::is_favorite(Uuid::new_v4(), auth(), pool).await {
        Ok(_) => panic!("is_favorite should fail"),
        Err(e) => assert_database_error_500(e, "is_favorite").await,
    }
}

#[tokio::test]
#[serial]
async fn wish_list_db_failures_are_500() {
    let pool = pool_without_tables().await;

    match handlers::wish_list::get_wish_list(auth(), pool.clone()).await {
        Ok(_) => panic!("get_wish_list should fail"),
        Err(e) => assert_database_error_500(e, "get_wish_list").await,
    }
    match handlers::wish_list::check_wish_list(Uuid::new_v4(), auth(), pool).await {
        Ok(_) => panic!("check_wish_list should fail"),
        Err(e) => assert_database_error_500(e, "check_wish_list").await,
    }
}

#[tokio::test]
#[serial]
async fn forgot_password_db_failure_is_500() {
    let pool = pool_without_tables().await;
    let request = ForgotPasswordRequest {
        email: "someone@example.com".to_string(),
    };
    match handlers::auth::forgot_password(request, pool).await {
        Ok(_) => panic!("forgot_password should fail"),
        Err(e) => assert_database_error_500(e, "forgot_password").await,
    }
}

#[tokio::test]
#[serial]
async fn setup_status_and_profile_db_failures_are_500() {
    let pool = pool_without_tables().await;

    match handlers::auth::get_setup_status(pool.clone()).await {
        Ok(_) => panic!("get_setup_status should fail"),
        Err(e) => assert_database_error_500(e, "get_setup_status").await,
    }
    match handlers::auth::get_current_user(auth(), pool.clone()).await {
        Ok(_) => panic!("get_current_user should fail"),
        Err(e) => assert_database_error_500(e, "get_current_user").await,
    }
    let request = humidor::models::ChangePasswordRequest {
        current_password: "a".to_string(),
        new_password: "b".to_string(),
    };
    match handlers::auth::change_password(request, auth(), pool).await {
        Ok(_) => panic!("change_password should fail"),
        Err(e) => assert_database_error_500(e, "change_password").await,
    }
}
