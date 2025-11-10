/// Record HTTP response metrics
pub fn record_response_metrics(
    path: &str,
    method: &str,
    status: u16,
    duration: std::time::Duration,
) {
    // Record response time
    metrics::histogram!(
        "http_request_duration_seconds",
        "method" => method.to_string(),
        "endpoint" => path.to_string(),
        "status" => status.to_string()
    )
    .record(duration.as_secs_f64());

    // Record status code distribution
    metrics::counter!(
        "http_responses_total",
        "method" => method.to_string(),
        "endpoint" => path.to_string(),
        "status" => status.to_string()
    )
    .increment(1);

    // Record error count separately
    if status >= 400 {
        metrics::counter!(
            "http_errors_total",
            "method" => method.to_string(),
            "endpoint" => path.to_string(),
            "status" => status.to_string()
        )
        .increment(1);
    }
}

/// Record database pool metrics
pub fn record_db_pool_metrics(size: usize, available: usize, max_size: usize) {
    metrics::gauge!("db_pool_connections_active").set((size - available) as f64);
    metrics::gauge!("db_pool_connections_idle").set(available as f64);
    metrics::gauge!("db_pool_connections_max").set(max_size as f64);
}
