use humidor::services::scrape_cigar_url;

#[tokio::test]
async fn scrape_refuses_internal_and_non_http_targets() {
    for url in [
        "http://127.0.0.1/",
        "http://[::1]/",
        "http://localhost/",
        "http://169.254.169.254/latest/meta-data/",
        "http://10.0.0.1/",
        "http://192.168.1.1/admin",
        "file:///etc/passwd",
        "http://93.184.216.34:8080/",
    ] {
        let err = scrape_cigar_url(url)
            .await
            .expect_err(&format!("{url} must be refused"))
            .to_string();
        assert!(
            err.contains("public address") || err.contains("allowed") || err.contains("DNS"),
            "unexpected error for {url}: {err}"
        );
    }
}
