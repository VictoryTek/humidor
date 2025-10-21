mod handlers;
mod models;

use std::{env, sync::Arc};
use tokio_postgres::{NoTls, Client};
use warp::{Filter, Reply};
use tracing_subscriber;

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

    // Run a simple migration
    client.execute(
        "CREATE TABLE IF NOT EXISTS cigars (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
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
            humidor_location VARCHAR,
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
        .or(delete_ring_gauge);

    // Root route
    let root = warp::path::end()
        .and(warp::get())
        .and_then(serve_index);

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);

    let routes = root
        .or(static_files)
        .or(api)
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
        Ok(content) => Ok(warp::reply::html(content)),
        Err(_) => Ok(warp::reply::html("<h1>Humidor Inventory</h1><p>Welcome to your cigar inventory system!</p>".to_string())),
    }
}
