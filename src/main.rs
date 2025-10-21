mod handlers;
mod models;

use std::{env, sync::Arc};
use tokio_postgres::{NoTls, Client};
use warp::{Filter, Reply};
use tracing_subscriber;

type DbPool = Arc<Client>;

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

    // Serve static files
    let static_files = warp::path("static")
        .and(warp::fs::dir("static"));

    // API routes
    let get_cigars = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("cigars"))
        .and(warp::get())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::cigars::get_cigars);

    let create_cigar = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("cigars"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::cigars::create_cigar);

    let api = get_cigars.or(create_cigar);

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
