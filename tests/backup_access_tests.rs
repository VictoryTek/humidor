mod common;

use common::*;
use humidor::errors::handle_rejection;
use humidor::routes::create_backup_routes;
use serial_test::serial;
use warp::Filter;
use warp::http::StatusCode;

const BOUNDARY: &str = "XBOUNDARYX";

fn multipart_body(filename: &str) -> Vec<u8> {
    format!(
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n\
         Content-Type: application/zip\r\n\r\nnot really a zip\r\n--{BOUNDARY}--\r\n"
    )
    .into_bytes()
}

async fn token_for(ctx: &TestContext, name: &str, is_admin: bool) -> String {
    let (id, username) = create_test_user(&ctx.pool, name, "Passw0rd!", is_admin)
        .await
        .expect("create user");
    create_test_jwt(id, &username).expect("jwt")
}

#[tokio::test]
#[serial]
async fn non_admin_and_anonymous_cannot_use_backup_endpoints() {
    let ctx = setup_test_db().await;
    let api = create_backup_routes(ctx.pool.clone()).recover(handle_rejection);
    let user = token_for(&ctx, "plain", false).await;

    let cases = [
        ("GET", "/api/v1/backups"),
        ("POST", "/api/v1/backups"),
        ("GET", "/api/v1/backups/x.zip/download"),
        ("DELETE", "/api/v1/backups/x.zip"),
        ("POST", "/api/v1/backups/x.zip/restore"),
        ("POST", "/api/v1/backups/upload"),
    ];
    for (method, path) in cases {
        let res = warp::test::request()
            .method(method)
            .path(path)
            .header("authorization", format!("Bearer {user}"))
            .reply(&api)
            .await;
        assert_eq!(
            res.status(),
            StatusCode::FORBIDDEN,
            "non-admin {method} {path}"
        );

        let res = warp::test::request()
            .method(method)
            .path(path)
            .reply(&api)
            .await;
        assert_eq!(
            res.status(),
            StatusCode::UNAUTHORIZED,
            "anon {method} {path}"
        );
    }
}

#[tokio::test]
#[serial]
async fn admin_can_list_backups() {
    let ctx = setup_test_db().await;
    let api = create_backup_routes(ctx.pool.clone()).recover(handle_rejection);
    let admin = token_for(&ctx, "adm", true).await;

    let res = warp::test::request()
        .path("/api/v1/backups")
        .header("authorization", format!("Bearer {admin}"))
        .reply(&api)
        .await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn admin_path_traversal_is_rejected() {
    let ctx = setup_test_db().await;
    let api = create_backup_routes(ctx.pool.clone()).recover(handle_rejection);
    let admin = token_for(&ctx, "adm", true).await;
    let auth = format!("Bearer {admin}");

    // Canary sits one level above `backups/`, i.e. where `../canary.zip` would resolve.
    let canary = std::path::Path::new("traversal_canary.zip");
    std::fs::write(canary, b"canary").unwrap();

    let res = warp::test::request()
        .path("/api/v1/backups/..%2Ftraversal_canary.zip/download")
        .header("authorization", &auth)
        .reply(&api)
        .await;
    assert_ne!(
        res.status(),
        StatusCode::OK,
        "download must not escape backups/"
    );
    assert!(!String::from_utf8_lossy(res.body()).contains("canary"));

    let _ = warp::test::request()
        .method("DELETE")
        .path("/api/v1/backups/..%2Ftraversal_canary.zip")
        .header("authorization", &auth)
        .reply(&api)
        .await;
    let survived = canary.exists();
    let _ = std::fs::remove_file(canary);
    assert!(survived, "delete must not escape backups/");

    // Upload with a traversal filename must not write outside backups/.
    let res = warp::test::request()
        .method("POST")
        .path("/api/v1/backups/upload")
        .header("authorization", &auth)
        .header(
            "content-type",
            format!("multipart/form-data; boundary={BOUNDARY}"),
        )
        .body(multipart_body("../traversal_upload.zip"))
        .reply(&api)
        .await;
    assert!(!std::path::Path::new("traversal_upload.zip").exists());
    let _ = std::fs::remove_file("traversal_upload.zip");
    assert!(String::from_utf8_lossy(res.body()).contains("Invalid filename"));
}

#[tokio::test]
#[serial]
async fn setup_restore_is_forbidden_once_an_admin_exists() {
    let ctx = setup_test_db().await;
    let api = create_backup_routes(ctx.pool.clone()).recover(handle_rejection);
    token_for(&ctx, "adm", true).await;

    let res = warp::test::request()
        .method("POST")
        .path("/api/v1/setup/restore")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={BOUNDARY}"),
        )
        .body(multipart_body("backup.zip"))
        .reply(&api)
        .await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

async fn admin_request<F, R>(
    api: &F,
    admin: &str,
    method: &str,
    path: &str,
) -> (StatusCode, serde_json::Value)
where
    F: Filter<Extract = (R,), Error = std::convert::Infallible> + Clone + 'static,
    R: warp::Reply + Send + 'static,
{
    let res = warp::test::request()
        .method(method)
        .path(path)
        .header("authorization", format!("Bearer {admin}"))
        .reply(api)
        .await;
    let body = serde_json::from_slice(res.body()).unwrap_or(serde_json::Value::Null);
    (res.status(), body)
}

#[tokio::test]
#[serial]
async fn backup_failures_return_real_error_statuses() {
    let ctx = setup_test_db().await;
    let api = create_backup_routes(ctx.pool.clone()).recover(handle_rejection);
    let admin = token_for(&ctx, "adm", true).await;

    // Missing backup: 404 for download, delete and restore (was 200 + message on delete/restore).
    for (method, path) in [
        ("GET", "/api/v1/backups/does_not_exist.zip/download"),
        ("DELETE", "/api/v1/backups/does_not_exist.zip"),
        ("POST", "/api/v1/backups/does_not_exist.zip/restore"),
    ] {
        let (status, body) = admin_request(&api, &admin, method, path).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{method} {path}");
        assert_eq!(body["error"], "NOT_FOUND");
    }

    // Invalid filename: 400 (was 200 + message).
    for (method, path) in [
        ("DELETE", "/api/v1/backups/notazip.txt"),
        ("POST", "/api/v1/backups/notazip.txt/restore"),
    ] {
        let (status, body) = admin_request(&api, &admin, method, path).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{method} {path}");
        assert_eq!(body["error"], "BAD_REQUEST");
    }
}

#[tokio::test]
#[serial]
async fn restoring_a_corrupt_backup_is_a_400_and_leaves_data_intact() {
    let ctx = setup_test_db().await;
    let api = create_backup_routes(ctx.pool.clone()).recover(handle_rejection);
    let admin = token_for(&ctx, "adm", true).await;

    std::fs::create_dir_all("backups").unwrap();
    let name = format!("corrupt_{}.zip", uuid::Uuid::new_v4());
    let path = std::path::Path::new("backups").join(&name);
    std::fs::write(&path, b"this is not a zip file").unwrap();

    let users_before: i64 = ctx
        .pool
        .get()
        .await
        .unwrap()
        .query_one("SELECT COUNT(*) FROM users", &[])
        .await
        .unwrap()
        .get(0);

    let (status, body) = admin_request(
        &api,
        &admin,
        "POST",
        &format!("/api/v1/backups/{name}/restore"),
    )
    .await;
    let _ = std::fs::remove_file(&path);

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["message"], "Invalid or corrupt backup file");
    // No internal detail (zip/parse error text, paths) reaches the client.
    assert!(!body.to_string().to_lowercase().contains("zip error"));
    assert!(!body.to_string().contains("backups/"));

    let users_after: i64 = ctx
        .pool
        .get()
        .await
        .unwrap()
        .query_one("SELECT COUNT(*) FROM users", &[])
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        users_before, users_after,
        "a rejected restore must not touch the database"
    );
}

#[tokio::test]
#[serial]
async fn upload_without_file_or_with_bad_name_is_a_400() {
    let ctx = setup_test_db().await;
    let api = create_backup_routes(ctx.pool.clone()).recover(handle_rejection);
    let admin = token_for(&ctx, "adm", true).await;
    let ct = format!("multipart/form-data; boundary={BOUNDARY}");

    let empty = format!("--{BOUNDARY}--\r\n").into_bytes();
    let res = warp::test::request()
        .method("POST")
        .path("/api/v1/backups/upload")
        .header("authorization", format!("Bearer {admin}"))
        .header("content-type", &ct)
        .body(empty)
        .reply(&api)
        .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let res = warp::test::request()
        .method("POST")
        .path("/api/v1/backups/upload")
        .header("authorization", format!("Bearer {admin}"))
        .header("content-type", &ct)
        .body(multipart_body("../evil.zip"))
        .reply(&api)
        .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert!(!std::path::Path::new("evil.zip").exists());
}
