/// CORS config for `CORS_MODE=permissive` (the default): any origin may call the API.
///
/// Credentials are deliberately NOT allowed. warp reflects the request's `Origin` header when
/// `allow_any_origin()` is used, so combining it with `Access-Control-Allow-Credentials: true`
/// would let any website make credentialed requests (e.g. with an `auth_token` cookie) and read
/// the responses. The frontend authenticates with an `Authorization` header, which needs no
/// credentialed CORS. Use `CORS_MODE=strict` with `ALLOWED_ORIGINS` for credentialed access.
pub fn permissive() -> warp::cors::Builder {
    warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
}

#[cfg(test)]
mod tests {
    use warp::Filter;

    #[tokio::test]
    async fn permissive_never_sends_allow_credentials() {
        let route = warp::any().map(|| "ok").with(super::permissive());

        let actual = warp::test::request()
            .method("GET")
            .header("origin", "https://evil.example")
            .reply(&route)
            .await;
        assert!(
            actual
                .headers()
                .get("access-control-allow-credentials")
                .is_none()
        );

        let preflight = warp::test::request()
            .method("OPTIONS")
            .header("origin", "https://evil.example")
            .header("access-control-request-method", "GET")
            .header("access-control-request-headers", "authorization")
            .reply(&route)
            .await;
        assert!(
            preflight
                .headers()
                .get("access-control-allow-credentials")
                .is_none()
        );
        // Non-credentialed cross-origin access is still permitted (self-hosted, dynamic IPs).
        assert!(
            preflight
                .headers()
                .get("access-control-allow-origin")
                .is_some()
        );
    }
}
