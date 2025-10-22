#![recursion_limit = "256"]

mod handlers;
mod models;
mod middleware;
mod errors;
mod validation;

use std::{env, sync::Arc};
use tokio_postgres::{NoTls, Client};
use warp::{Filter, Reply};
use tracing_subscriber;
use middleware::{with_current_user, handle_rejection};

type DbPool = Arc<Client>;

#[derive(Debug)]
struct InvalidUuid;
impl warp::reject::Reject for InvalidUuid {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://humidor_user:humidor_pass@localhost:5432/humidor_db".to_string());

    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;
    
    // Spawn the connection in a background task
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    // Run database migrations
    // Create users table
    client.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            username VARCHAR(50) UNIQUE NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL,
            full_name VARCHAR(255) NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            is_admin BOOLEAN NOT NULL DEFAULT false,
            is_active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        &[],
    ).await?;

    // Create humidors table
    client.execute(
        "CREATE TABLE IF NOT EXISTS humidors (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            name VARCHAR(255) NOT NULL,
            description TEXT,
            capacity INTEGER,
            target_humidity INTEGER,
            location VARCHAR(255),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        &[],
    ).await?;

    // Create cigars table (updated with humidor_id)
    client.execute(
        "CREATE TABLE IF NOT EXISTS cigars (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            humidor_id UUID REFERENCES humidors(id) ON DELETE SET NULL,
            brand VARCHAR NOT NULL,
            name VARCHAR NOT NULL,
            size VARCHAR NOT NULL,
            strength VARCHAR NOT NULL,
            origin VARCHAR NOT NULL,
            wrapper VARCHAR,
            binder VARCHAR,
            filler VARCHAR,
            price DECIMAL(10,2),
            purchase_date TIMESTAMPTZ,
            notes TEXT,
            quantity INTEGER NOT NULL DEFAULT 1,
            ring_gauge INTEGER,
            length DECIMAL(4,2),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        &[],
    ).await?;

    let db_pool = Arc::new(client);
    
    // Helper function to pass database to handlers
    fn with_db(db: DbPool) -> impl Filter<Extract = (DbPool,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }

    // Helper function to extract UUID from path
    fn with_uuid() -> impl Filter<Extract = (uuid::Uuid,), Error = warp::Rejection> + Copy {
        warp::path::param::<String>()
            .and_then(|id: String| async move {
                uuid::Uuid::parse_str(&id)
                    .map_err(|_| warp::reject::custom(InvalidUuid))
            })
    }

    // Serve static files
    let static_files = warp::path("static")
        .and(warp::fs::dir("static"));

    // Cigar API routes
    let get_cigars = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("cigars"))
        .and(warp::get())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_cigars);

    let create_cigar = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("cigars"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::create_cigar);

    let get_cigar = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("cigars"))
        .and(with_uuid())
        .and(warp::get())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::get_cigar);

    let update_cigar = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("cigars"))
        .and(with_uuid())
        .and(warp::put())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::update_cigar);

    let delete_cigar = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("cigars"))
        .and(with_uuid())
        .and(warp::delete())
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

    // Combine all API routes
    let api = get_cigars
        .or(create_cigar)
        .or(get_cigar)
        .or(update_cigar)
        .or(delete_cigar)
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
        .or(get_humidors)
        .or(get_humidor)
        .or(create_humidor)
        .or(update_humidor)
        .or(delete_humidor)
        .or(get_humidor_cigars);

    // Root route
    let root = warp::path::end()
        .and(warp::get())
        .and_then(serve_index);

    // Setup route
    let setup = warp::path("setup.html")
        .and(warp::get())
        .and_then(serve_setup);

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);

    let routes = root
        .or(setup)
        .or(static_files)
        .or(api)
        .recover(handle_rejection)
        .with(cors);

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    println!("Server running on http://0.0.0.0:{}", port);
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;

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
        },
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
        ).into_response()),
    }
}
