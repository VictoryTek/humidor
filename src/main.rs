#![recursion_limit = "256"]

mod errors;
mod handlers;
mod middleware;
mod models;
mod services;
mod validation;

use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use middleware::{handle_rejection, with_current_user};
use refinery::embed_migrations;
use std::env;
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

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://humidor_user:humidor_pass@localhost:5432/humidor_db".to_string()
    });

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

    let login_user = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("auth"))
        .and(warp::path("login"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::login_user);

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

    // Combine all API routes
    let api = scrape_cigar
        .or(create_cigar)
        .or(update_cigar)
        .or(delete_cigar)
        .or(get_cigar)
        .or(get_cigars)
        .or(get_brands)
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
        .or(get_setup_status)
        .or(create_setup_user)
        .or(login_user)
        .or(get_current_user)
        .or(update_current_user)
        .or(change_password)
        .or(get_humidors)
        .or(get_humidor_cigars) // Must come before get_humidor (more specific route)
        .or(create_humidor)
        .or(update_humidor)
        .or(delete_humidor)
        .or(get_humidor) // Less specific, should be last
        .or(check_favorite) // Must come before remove_favorite (more specific route)
        .or(get_favorites)
        .or(add_favorite)
        .or(remove_favorite);

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

    let routes = root
        .or(setup)
        .or(login)
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
