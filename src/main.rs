#![recursion_limit = "256"]

mod errors;
mod handlers;
mod middleware;
mod models;
mod services;
mod validation;

use anyhow::{anyhow, bail};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use middleware::{handle_rejection, with_current_user};
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

#[derive(Debug)]
struct InvalidUuid;
impl warp::reject::Reject for InvalidUuid {}

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

    // Helper function to pass database pool to handlers
    fn with_db(
        db: DbPool,
    ) -> impl Filter<Extract = (DbPool,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }

    // Helper function to extract UUID from path
    fn with_uuid() -> impl Filter<Extract = (uuid::Uuid,), Error = warp::Rejection> + Copy {
        warp::path::param::<String>().and_then(|id: String| async move {
            uuid::Uuid::parse_str(&id).map_err(|_| warp::reject::custom(InvalidUuid))
        })
    }

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

    // Cigar API routes (authenticated)
    let get_cigars = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("cigars"))
        .and(warp::path::end())
        .and(warp::get())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_cigars);

    let create_cigar = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("cigars"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::create_cigar);

    let scrape_cigar = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("cigars"))
        .and(warp::path("scrape"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_current_user(db_pool.clone()))
        .and_then(handlers::scrape_cigar_url);

    let get_cigar = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("cigars"))
        .and(with_uuid())
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_cigar);

    let update_cigar = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("cigars"))
        .and(with_uuid())
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_cigar);

    let delete_cigar = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("cigars"))
        .and(with_uuid())
        .and(warp::path::end())
        .and(warp::delete())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::delete_cigar);

    // Brand API routes
    let get_brands = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("brands"))
        .and(warp::get())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_brands);

    let create_brand = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("brands"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::create_brand);

    let update_brand = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("brands"))
        .and(with_uuid())
        .and(warp::put())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_brand);

    let delete_brand = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("brands"))
        .and(with_uuid())
        .and(warp::delete())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::delete_brand);

    // Size API routes
    let get_sizes = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("sizes"))
        .and(warp::get())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_sizes);

    let create_size = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("sizes"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::create_size);

    let update_size = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("sizes"))
        .and(with_uuid())
        .and(warp::put())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_size);

    let delete_size = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("sizes"))
        .and(with_uuid())
        .and(warp::delete())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::delete_size);

    // Origin API routes
    let get_origins = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("origins"))
        .and(warp::get())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_origins);

    let create_origin = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("origins"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::create_origin);

    let update_origin = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("origins"))
        .and(with_uuid())
        .and(warp::put())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_origin);

    let delete_origin = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("origins"))
        .and(with_uuid())
        .and(warp::delete())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::delete_origin);

    // Strength API routes
    let get_strengths = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("strengths"))
        .and(warp::get())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_strengths);

    let create_strength = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("strengths"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::create_strength);

    let update_strength = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("strengths"))
        .and(with_uuid())
        .and(warp::put())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_strength);

    let delete_strength = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("strengths"))
        .and(with_uuid())
        .and(warp::delete())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::delete_strength);

    // Ring Gauge API routes
    let get_ring_gauges = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("ring-gauges"))
        .and(warp::get())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_ring_gauges);

    let create_ring_gauge = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("ring-gauges"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::create_ring_gauge);

    let update_ring_gauge = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("ring-gauges"))
        .and(with_uuid())
        .and(warp::put())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_ring_gauge);

    let delete_ring_gauge = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("ring-gauges"))
        .and(with_uuid())
        .and(warp::delete())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::delete_ring_gauge);

    // Authentication and Setup routes
    let get_setup_status = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("setup"))
        .and(warp::path("status"))
        .and(warp::get())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_setup_status);

    let create_setup_user = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("setup"))
        .and(warp::path("user"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::create_setup_user);

    let setup_restore = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("setup"))
        .and(warp::path("restore"))
        .and(warp::post())
        .and(warp::multipart::form().max_length(100_000_000)) // 100MB max
        .and(with_db(db_pool.clone()))
        .and_then(handlers::backups::setup_restore_backup);

    let login_user = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("auth"))
        .and(warp::path("login"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::login_user);

    // Password reset routes (public)
    let forgot_password = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("auth"))
        .and(warp::path("forgot-password"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::forgot_password);

    let reset_password = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("auth"))
        .and(warp::path("reset-password"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::reset_password);

    let email_config_status = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("auth"))
        .and(warp::path("email-config"))
        .and(warp::get())
        .and_then(handlers::check_email_config);

    // User profile API routes (authenticated)
    let get_current_user = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("users"))
        .and(warp::path("self"))
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_current_user);

    let update_current_user = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("users"))
        .and(warp::path("self"))
        .and(warp::put())
        .and(warp::body::json())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_current_user);

    let change_password = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("users"))
        .and(warp::path("password"))
        .and(warp::put())
        .and(warp::body::json())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::change_password);

    // Backup/Restore API routes (authenticated)
    let get_backups = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("backups"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::backups::get_backups);

    let create_backup = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("backups"))
        .and(warp::path::end())
        .and(warp::post())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::backups::create_backup_handler);

    let download_backup = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("backups"))
        .and(warp::path::param())
        .and(warp::path("download"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::backups::download_backup);

    let delete_backup = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("backups"))
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::delete())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::backups::delete_backup_handler);

    let restore_backup = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("backups"))
        .and(warp::path::param())
        .and(warp::path("restore"))
        .and(warp::path::end())
        .and(warp::post())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::backups::restore_backup_handler);

    let upload_backup = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("backups"))
        .and(warp::path("upload"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::multipart::form().max_length(100_000_000)) // 100MB max
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::backups::upload_backup);

    // Humidor API routes (authenticated)
    let get_humidors = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("humidors"))
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_humidors);

    let get_humidor = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("humidors"))
        .and(with_uuid())
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_humidor);

    let create_humidor = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("humidors"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::create_humidor);

    let update_humidor = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("humidors"))
        .and(with_uuid())
        .and(warp::put())
        .and(warp::body::json())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_humidor);

    let delete_humidor = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("humidors"))
        .and(with_uuid())
        .and(warp::delete())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::delete_humidor);

    let get_humidor_cigars = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("humidors"))
        .and(with_uuid())
        .and(warp::path("cigars"))
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_humidor_cigars);

    // Favorites routes
    let get_favorites = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("favorites"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_favorites);

    let add_favorite = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("favorites"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::add_favorite);

    let remove_favorite = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("favorites"))
        .and(with_uuid())
        .and(warp::path::end())
        .and(warp::delete())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::remove_favorite);

    let check_favorite = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("favorites"))
        .and(with_uuid())
        .and(warp::path("check"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::is_favorite);

    // Wish List routes
    let get_wish_list = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("wish_list"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_wish_list);

    let add_to_wish_list = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("wish_list"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::add_to_wish_list);

    let remove_from_wish_list = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("wish_list"))
        .and(with_uuid())
        .and(warp::path::end())
        .and(warp::delete())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::remove_from_wish_list);

    let check_wish_list = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("wish_list"))
        .and(with_uuid())
        .and(warp::path("check"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::check_wish_list);

    let update_wish_list_notes = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("wish_list"))
        .and(with_uuid())
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(with_current_user(db_pool.clone()))
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_wish_list_notes);

    // Combine all API routes
    // Group routes to reduce type complexity
    let backup_routes = get_backups
        .or(create_backup)
        .or(download_backup)
        .or(delete_backup)
        .or(restore_backup)
        .or(upload_backup)
        .boxed();
    
    let cigar_routes = scrape_cigar
        .or(create_cigar)
        .or(update_cigar)
        .or(delete_cigar)
        .or(get_cigar)
        .or(get_cigars)
        .boxed();
    
    let organizer_routes = get_brands
        .or(create_brand)
        .or(update_brand)
        .or(delete_brand)
        .or(get_sizes)
        .or(create_size)
        .or(update_size)
        .or(delete_size)
        .or(get_origins)
        .or(create_origin)
        .or(update_origin)
        .or(delete_origin)
        .or(get_strengths)
        .or(create_strength)
        .or(update_strength)
        .or(delete_strength)
        .or(get_ring_gauges)
        .or(create_ring_gauge)
        .or(update_ring_gauge)
        .or(delete_ring_gauge)
        .boxed();
    
    let auth_routes = get_setup_status
        .or(create_setup_user)
        .or(login_user)
        .or(forgot_password)
        .or(reset_password)
        .or(email_config_status)
        .or(get_current_user)
        .or(update_current_user)
        .or(change_password)
        .boxed();
    
    let humidor_routes = get_humidors
        .or(get_humidor_cigars) // Must come before get_humidor (more specific route)
        .or(create_humidor)
        .or(update_humidor)
        .or(delete_humidor)
        .or(get_humidor) // Less specific, should be last
        .boxed();
    
    let favorite_routes = check_favorite // Must come before remove_favorite (more specific route)
        .or(get_favorites)
        .or(add_favorite)
        .or(remove_favorite)
        .or(check_wish_list) // Must come before remove_from_wish_list (more specific route)
        .or(update_wish_list_notes) // Must come before remove_from_wish_list (both have UUID path)
        .or(get_wish_list)
        .or(add_to_wish_list)
        .or(remove_from_wish_list)
        .boxed();
    
    // Combine all routes
    let api = backup_routes
        .or(cigar_routes)
        .or(organizer_routes)
        .or(auth_routes)
        .or(setup_restore)
        .or(humidor_routes)
        .or(favorite_routes);

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

    println!("CORS: Allowing origins: {:?}", allowed_origins);

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
        .with(cors);

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    tracing::info!(
        addr = %format!("0.0.0.0:{}", port),
        port = port,
        "Server started successfully, listening for connections"
    );
    
    println!("Server running on http://0.0.0.0:{}", port);

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
