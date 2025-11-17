use crate::DbPool;
use crate::middleware::RateLimiter;
use std::net::IpAddr;
use warp::Filter;

// Re-export commonly used middleware and types
pub use crate::middleware::with_current_user;

#[derive(Debug)]
pub struct InvalidUuid;
impl warp::reject::Reject for InvalidUuid {}

/// Helper function to pass database pool to handlers
pub fn with_db(
    db: DbPool,
) -> impl Filter<Extract = (DbPool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

/// Helper function to pass rate limiter to handlers
pub fn with_rate_limiter(
    limiter: RateLimiter,
) -> impl Filter<Extract = (RateLimiter,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || limiter.clone())
}

/// Helper function to extract client IP address
pub fn with_client_ip()
-> impl Filter<Extract = (Option<IpAddr>,), Error = std::convert::Infallible> + Clone {
    warp::addr::remote().map(|addr: Option<std::net::SocketAddr>| addr.map(|socket| socket.ip()))
}

/// Helper function to extract UUID from path
pub fn with_uuid() -> impl Filter<Extract = (uuid::Uuid,), Error = warp::Rejection> + Copy {
    warp::path::param::<String>().and_then(|id: String| async move {
        uuid::Uuid::parse_str(&id).map_err(|_| warp::reject::custom(InvalidUuid))
    })
}

/// Helper function to parse JSON body with size limit
/// Default limit: 1MB for JSON payloads (reasonable for API requests)
pub fn json_body<T: Send + serde::de::DeserializeOwned>()
-> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 1024) // 1MB
        .and(warp::body::json())
}
