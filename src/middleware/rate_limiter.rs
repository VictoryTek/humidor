use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

/// Rate limiter for authentication attempts
/// Uses a sliding window algorithm to track failed login attempts per IP address
#[derive(Clone)]
pub struct RateLimiter {
    /// Map of IP address to list of attempt timestamps
    attempts: Arc<RwLock<HashMap<IpAddr, Vec<SystemTime>>>>,
    /// Maximum number of failed attempts allowed within the time window
    max_attempts: usize,
    /// Time window in seconds for tracking attempts
    window_secs: u64,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration
    /// Default: 5 attempts per 15 minutes (900 seconds)
    pub fn new(max_attempts: usize, window_secs: u64) -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
            max_attempts,
            window_secs,
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(5, 900)
    }
}

impl RateLimiter {
    /// Check if an IP address is currently rate limited
    /// Returns true if the IP has exceeded the rate limit
    pub async fn is_rate_limited(&self, ip: IpAddr) -> bool {
        let mut attempts = self.attempts.write().await;

        // Get or create the attempt list for this IP
        let ip_attempts = attempts.entry(ip).or_insert_with(Vec::new);

        // Remove expired attempts (outside the time window)
        let cutoff_time = SystemTime::now() - Duration::from_secs(self.window_secs);
        ip_attempts.retain(|&attempt_time| attempt_time > cutoff_time);

        // Check if we've exceeded the limit
        ip_attempts.len() >= self.max_attempts
    }

    /// Record a failed login attempt for an IP address
    pub async fn record_attempt(&self, ip: IpAddr) {
        let mut attempts = self.attempts.write().await;

        // Get or create the attempt list for this IP
        let ip_attempts = attempts.entry(ip).or_insert_with(Vec::new);

        // Add the current attempt
        ip_attempts.push(SystemTime::now());

        tracing::debug!(
            ip = %ip,
            attempt_count = ip_attempts.len(),
            "Recorded failed login attempt"
        );
    }

    /// Clear rate limit records for an IP address (called on successful login)
    pub async fn clear_attempts(&self, ip: IpAddr) {
        let mut attempts = self.attempts.write().await;

        if attempts.remove(&ip).is_some() {
            tracing::debug!(ip = %ip, "Cleared rate limit records after successful login");
        }
    }

    /// Cleanup task to periodically remove expired entries
    /// Should be spawned as a background task
    pub async fn cleanup_expired(&self) {
        let mut attempts = self.attempts.write().await;
        let cutoff_time = SystemTime::now() - Duration::from_secs(self.window_secs);

        // Remove IPs with no recent attempts
        attempts.retain(|_, ip_attempts| {
            ip_attempts.retain(|&attempt_time| attempt_time > cutoff_time);
            !ip_attempts.is_empty()
        });

        tracing::debug!(
            remaining_ips = attempts.len(),
            "Cleaned up expired rate limit entries"
        );
    }

    /// Spawn a background cleanup task that runs periodically
    /// Cleans up expired entries every 5 minutes
    pub fn spawn_cleanup_task(self) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes

            loop {
                interval.tick().await;
                self.cleanup_expired().await;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(3, 10); // 3 attempts in 10 seconds
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Should not be limited initially
        assert!(!limiter.is_rate_limited(ip).await);

        // Record 3 attempts
        for _ in 0..3 {
            limiter.record_attempt(ip).await;
        }

        // Should now be limited
        assert!(limiter.is_rate_limited(ip).await);

        // Clear and should not be limited
        limiter.clear_attempts(ip).await;
        assert!(!limiter.is_rate_limited(ip).await);
    }

    #[tokio::test]
    async fn test_rate_limiter_expiry() {
        let limiter = RateLimiter::new(2, 1); // 2 attempts in 1 second
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Record 2 attempts
        limiter.record_attempt(ip).await;
        limiter.record_attempt(ip).await;

        // Should be limited
        assert!(limiter.is_rate_limited(ip).await);

        // Wait for expiry
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should not be limited anymore
        assert!(!limiter.is_rate_limited(ip).await);
    }
}
