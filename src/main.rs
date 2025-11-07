#![recursion_limit = "256"]

mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;
mod validation;

use anyhow::{anyhow, bail};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use middleware::{handle_rejection, RateLimiter};
use refinery::embed_migrations;
use std::env;
use std::fs;
use tokio_postgres::NoTls;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use warp::{Filter, Reply};
use warp::log;

// Embed migrations from the migrations directory
embed_migrations!("migrations");

type DbPool = Pool;

/// Read a secret from Docker secrets or fall back to environment variable
/// Docker secrets are mounted at /run/secrets/<secret_name>
fn read_secret(secret_name: &str, env_var: &str) -> Option<String> {
    let secret_path = format!("/run/secrets/{}", secret_name);
    
    // Try Docker secret file first
    if let Ok(content) = fs::read_to_string(&secret_path) {
        tracing::debug!(
            secret_name = secret_name,
            source = "docker_secret",
            "Successfully read secret from file"
        );
        return Some(content.trim().to_string());
    }
    
    // Fall back to environment variable
    if let Ok(value) = env::var(env_var) {
        tracing::debug!(
            secret_name = secret_name,
            env_var = env_var,
            source = "environment",
            "Successfully read secret from environment"
        );
        return Some(value);
    }
    
    tracing::warn!(
        secret_name = secret_name,
        env_var = env_var,
        "Failed to read secret from both Docker secrets and environment"
    );
    None
}

/// Validate JWT secret at startup - fail fast before accepting requests
fn validate_jwt_secret() -> anyhow::Result<()> {
    let secret = read_secret("jwt_secret", "JWT_SECRET")
        .ok_or_else(|| {
            anyhow!(
                "JWT_SECRET not found in /run/secrets/jwt_secret or JWT_SECRET environment variable. \
                 Generate a secure secret with: openssl rand -base64 32"
            )
        })?;
    
    // Validate minimum length for cryptographic security
    if secret.len() < 32 {
        bail!(
            "JWT_SECRET must be at least 32 characters for cryptographic security. \
             Current length: {}. Generate a secure secret with: openssl rand -base64 32",
            secret.len()
        );
    }
    
    tracing::info!(
        secret_length = secret.len(),
        "JWT secret validated successfully"
    );
    
    Ok(())
}

/// Validate database connection at startup - fail fast if database is unreachable
async fn validate_database_connection(pool: &DbPool) -> anyhow::Result<()> {
    match pool.get().await {
        Ok(client) => {
            // Test query to verify database is actually working
            match client.query_one("SELECT 1 as test", &[]).await {
                Ok(_) => {
                    tracing::info!("Database connection validated successfully");
                    Ok(())
                }
                Err(e) => {
                    bail!(
                        "Database connection test query failed: {}. \
                         Verify database is running and schema is initialized.",
                        e
                    );
                }
            }
        }
        Err(e) => {
            bail!(
                "Failed to acquire database connection from pool: {}. \
                 Check DATABASE_URL configuration and verify PostgreSQL is running.",
                e
            );
        }
    }
}

/// Validate SMTP configuration if email service is enabled
fn validate_smtp_config() -> anyhow::Result<()> {
    // Check if SMTP is intended to be used
    let smtp_enabled = env::var("SMTP_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase() == "true";
    
    if !smtp_enabled {
        tracing::info!("SMTP email service disabled (SMTP_ENABLED=false or not set)");
        return Ok(());
    }
    
    // If SMTP is enabled, validate required configuration
    let mut missing = Vec::new();
    
    if env::var("SMTP_HOST").is_err() {
        missing.push("SMTP_HOST");
    }
    if env::var("SMTP_PORT").is_err() {
        missing.push("SMTP_PORT");
    }
    if env::var("SMTP_USERNAME").is_err() {
        missing.push("SMTP_USERNAME");
    }
    if env::var("SMTP_PASSWORD").is_err() {
        missing.push("SMTP_PASSWORD");
    }
    if env::var("SMTP_FROM").is_err() {
        missing.push("SMTP_FROM");
    }
    
    if !missing.is_empty() {
        bail!(
            "SMTP is enabled but required configuration is missing: {}. \
             Either set SMTP_ENABLED=false or provide all SMTP configuration variables.",
            missing.join(", ")
        );
    }
    
    tracing::info!(
        smtp_host = env::var("SMTP_HOST").unwrap(),
        smtp_port = env::var("SMTP_PORT").unwrap(),
        smtp_from = env::var("SMTP_FROM").unwrap(),
        "SMTP configuration validated successfully"
    );
    
    Ok(())
}

/// Comprehensive startup configuration validation - fail fast with clear errors
async fn validate_environment(pool: &DbPool) -> anyhow::Result<()> {
    tracing::info!("Starting environment validation...");
    
    // Validate JWT secret
    tracing::debug!("Validating JWT secret configuration...");
    validate_jwt_secret()?;
    
    // Validate database connectivity
    tracing::debug!("Validating database connection...");
    validate_database_connection(pool).await?;
    
    // Validate SMTP configuration if enabled
    tracing::debug!("Validating SMTP configuration...");
    validate_smtp_config()?;
    
    tracing::info!("✅ All environment validations passed successfully");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Enhanced structured logging with JSON format
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "humidor=info,warp=info,refinery=info".into())
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
                .json()
        )
        .init();

    tracing::info!(
        app_name = "humidor",
        version = env!("CARGO_PKG_VERSION"),
        "Starting Humidor application"
    );

    // Build DATABASE_URL from secrets or environment
    let database_url = if let Some(template) = env::var("DATABASE_URL_TEMPLATE").ok() {
        // Using Docker secrets - read username and password from secret files
        let db_user = read_secret("db_user", "DB_USER")
            .unwrap_or_else(|| "humidor_user".to_string());
        let db_password = read_secret("db_password", "DB_PASSWORD")
            .unwrap_or_else(|| "humidor_pass".to_string());
        
        template
            .replace("{{DB_USER}}", &db_user)
            .replace("{{DB_PASSWORD}}", &db_password)
    } else {
        // Fall back to DATABASE_URL environment variable or default
        env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://humidor_user:humidor_pass@localhost:5432/humidor_db".to_string()
        })
    };

    // Create connection pool configuration
    tracing::info!(
        max_connections = 20,
        recycling_method = "Fast",
        "Creating database connection pool"
    );
    
    let mut config = Config::new();
    config.url = Some(database_url.clone());
    config.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    // Create the pool with a maximum of 20 connections
    let pool = config.create_pool(Some(Runtime::Tokio1), NoTls)?;

    // Test the connection and run migrations
    let mut client = pool.get().await?;
    tracing::info!(
        pool_status = "connected",
        "Database connection pool created successfully"
    );

    // Run database migrations using refinery
    tracing::info!("Running database migrations...");
    match migrations::runner().run_async(&mut **client).await {
        Ok(report) => {
            tracing::info!(
                applied_migrations = report.applied_migrations().len(),
                "Database migrations completed successfully"
            );
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                "Database migrations failed"
            );
            return Err(e.into());
        }
    }

    // Drop the migration client back to the pool
    drop(client);

    // Use the pool for all handlers
    let db_pool = pool;

    // Validate all environment configuration before accepting requests
    // This ensures the application fails fast with clear error messages
    // if any required configuration is missing or invalid
    validate_environment(&db_pool).await?;

    // Get server port from environment
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9898);

    tracing::info!(
        port = port,
        environment = env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()),
        "Configuring server"
    );

    // Initialize rate limiter for authentication (5 attempts per 15 minutes)
    let rate_limiter = RateLimiter::default();
    
    // Spawn cleanup task to remove expired rate limit entries
    rate_limiter.clone().spawn_cleanup_task();
    
    tracing::info!(
        max_attempts = 5,
        window_minutes = 15,
        "Rate limiter initialized for authentication endpoints"
    );

    // Spawn background task to clean up expired password reset tokens
    let cleanup_pool = db_pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // Run every hour
        
        loop {
            interval.tick().await;
            
            match cleanup_pool.get().await {
                Ok(client) => {
                    // Delete tokens older than 30 minutes
                    let result = client
                        .execute(
                            "DELETE FROM password_reset_tokens WHERE created_at < NOW() - INTERVAL '30 minutes'",
                            &[],
                        )
                        .await;
                    
                    match result {
                        Ok(deleted_count) => {
                            if deleted_count > 0 {
                                tracing::info!(
                                    deleted_tokens = deleted_count,
                                    "Cleaned up expired password reset tokens"
                                );
                            } else {
                                tracing::debug!("No expired password reset tokens to clean up");
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                error = %e,
                                "Failed to clean up expired password reset tokens"
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        "Failed to get database connection for token cleanup"
                    );
                }
            }
        }
    });
    
    tracing::info!(
        cleanup_interval_minutes = 60,
        token_expiration_minutes = 30,
        "Password reset token cleanup task initialized"
    );

    // Request logging middleware with structured logging
    fn log_requests() -> log::Log<impl Fn(log::Info) + Copy> {
        warp::log::custom(|info| {
            tracing::info!(
                method = %info.method(),
                path = %info.path(),
                status = %info.status().as_u16(),
                duration_ms = %info.elapsed().as_millis(),
                remote_addr = ?info.remote_addr(),
                "request completed"
            );
        })
    }

    // Serve static files
    let static_files = warp::path("static").and(warp::fs::dir("static"));

    // Create all API routes using route modules
    let auth_routes = routes::create_auth_routes(db_pool.clone(), rate_limiter.clone()).boxed();
    let user_routes = routes::create_user_routes(db_pool.clone()).boxed();
    let cigar_routes = routes::create_cigar_routes(db_pool.clone()).boxed();
    let organizer_routes = routes::create_organizer_routes(db_pool.clone()).boxed();
    let humidor_routes = routes::create_humidor_routes(db_pool.clone()).boxed();
    let favorite_routes = routes::create_favorite_routes(db_pool.clone()).boxed();
    let backup_routes = routes::create_backup_routes(db_pool.clone()).boxed();

    // Combine all API routes
    let api = auth_routes
        .or(user_routes)
        .or(cigar_routes)
        .or(organizer_routes)
        .or(humidor_routes)
        .or(favorite_routes)
        .or(backup_routes);

    // Health check endpoint (no auth required)
    let health = warp::path("health")
        .and(warp::get())
        .map(|| {
            warp::reply::json(&serde_json::json!({
                "status": "ok",
                "service": "humidor"
            }))
        });

    // Root route
    let root = warp::path::end().and(warp::get()).and_then(serve_index);

    // Setup route
    let setup = warp::path("setup.html")
        .and(warp::get())
        .and_then(serve_setup);

    // Login route
    let login = warp::path("login.html")
        .and(warp::get())
        .and_then(serve_login);

    // Password reset page routes
    let forgot_password_page = warp::path("forgot-password.html")
        .and(warp::get())
        .and_then(serve_forgot_password);

    let reset_password_page = warp::path("reset-password.html")
        .and(warp::get())
        .and_then(serve_reset_password);

    // Configure CORS - restrictive by default for security
    // Use ALLOWED_ORIGINS env var for production (comma-separated list)
    let allowed_origins: Vec<String> = env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:9898,http://127.0.0.1:9898".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    tracing::info!(
        allowed_origins = ?allowed_origins,
        "CORS configuration loaded"
    );

    let cors = warp::cors()
        .allow_origins(allowed_origins.iter().map(|s| s.as_str()))
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allow_credentials(true); // Required for cookie-based auth

    let routes = health
        .or(root)
        .or(setup)
        .or(login)
        .or(forgot_password_page)
        .or(reset_password_page)
        .or(static_files)
        .or(api)
        .with(log_requests())
        .recover(handle_rejection)
        .with(cors)
        .map(|reply| warp::reply::with_header(reply, "Strict-Transport-Security", "max-age=31536000; includeSubDomains; preload"))
        .map(|reply| warp::reply::with_header(reply, "X-Content-Type-Options", "nosniff"))
        .map(|reply| warp::reply::with_header(reply, "X-Frame-Options", "DENY"))
        .map(|reply| warp::reply::with_header(reply, "X-XSS-Protection", "1; mode=block"))
        .map(|reply| warp::reply::with_header(reply, "Content-Security-Policy", "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'"))
        .map(|reply| warp::reply::with_header(reply, "Referrer-Policy", "no-referrer-when-downgrade"))
        .map(|reply| warp::reply::with_header(reply, "Permissions-Policy", "geolocation=(), microphone=(), camera=()"));

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    tracing::info!(
        addr = %format!("0.0.0.0:{}", port),
        port = port,
        url = %format!("http://0.0.0.0:{}", port),
        "Server started successfully, listening for connections"
    );

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;

    Ok(())
}

async fn serve_index() -> Result<impl Reply, warp::Rejection> {
    match tokio::fs::read_to_string("static/index.html").await {
        Ok(content) => {
            // Inject setup check script into the HTML
            let setup_script = r#"
<script>
// Check if setup is needed and redirect to setup page
fetch('/api/v1/setup/status')
    .then(response => response.json())
    .then(data => {
        if (data.needs_setup) {
            // Only redirect if we're not already on the setup page
            if (!window.location.pathname.includes('setup.html')) {
                window.location.href = '/setup.html';
            }
        }
    })
    .catch(error => {
        console.error('Failed to check setup status:', error);
    });
</script>
"#;

            // Insert the script before the closing </body> tag
            let modified_content = content.replace("</body>", &format!("{}</body>", setup_script));
            Ok(warp::reply::html(modified_content))
        }
        Err(_) => {
            // Fallback content with setup check
            let fallback_html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Humidor Inventory</title>
</head>
<body>
    <h1>Humidor Inventory</h1>
    <p>Welcome to your cigar inventory system!</p>
    <script>
    fetch('/api/v1/setup/status')
        .then(response => response.json())
        .then(data => {
            if (data.needs_setup) {
                window.location.href = '/setup.html';
            }
        })
        .catch(error => {
            console.error('Failed to check setup status:', error);
        });
    </script>
</body>
</html>
"#;
            Ok(warp::reply::html(fallback_html.to_string()))
        }
    }
}

async fn serve_setup() -> Result<impl Reply, warp::Rejection> {
    match tokio::fs::read_to_string("static/setup.html").await {
        Ok(content) => Ok(warp::reply::html(content).into_response()),
        Err(_) => Ok(warp::reply::with_status(
            warp::reply::html("<h1>Setup Not Found</h1>".to_string()),
            warp::http::StatusCode::NOT_FOUND,
        )
        .into_response()),
    }
}

async fn serve_login() -> Result<impl Reply, warp::Rejection> {
    match tokio::fs::read_to_string("static/login.html").await {
        Ok(content) => Ok(warp::reply::html(content).into_response()),
        Err(_) => Ok(warp::reply::with_status(
            warp::reply::html("<h1>Login Not Found</h1>".to_string()),
            warp::http::StatusCode::NOT_FOUND,
        )
        .into_response()),
    }
}

async fn serve_forgot_password() -> Result<impl Reply, warp::Rejection> {
    match tokio::fs::read_to_string("static/forgot-password.html").await {
        Ok(content) => Ok(warp::reply::html(content).into_response()),
        Err(_) => Ok(warp::reply::with_status(
            warp::reply::html("<h1>Forgot Password Not Found</h1>".to_string()),
            warp::http::StatusCode::NOT_FOUND,
        )
        .into_response()),
    }
}

async fn serve_reset_password() -> Result<impl Reply, warp::Rejection> {
    match tokio::fs::read_to_string("static/reset-password.html").await {
        Ok(content) => Ok(warp::reply::html(content).into_response()),
        Err(_) => Ok(warp::reply::with_status(
            warp::reply::html("<h1>Reset Password Not Found</h1>".to_string()),
            warp::http::StatusCode::NOT_FOUND,
        )
        .into_response()),
    }
}
