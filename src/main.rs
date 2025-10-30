#![recursion_limit = "256"]

mod handlers;
mod models;
mod middleware;
mod errors;
mod validation;
mod services;

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

    // Create organizer tables FIRST (before cigars table that references them)
    // Brands table
    client.execute(
        "CREATE TABLE IF NOT EXISTS brands (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR NOT NULL UNIQUE,
            description TEXT,
            country VARCHAR,
            website VARCHAR,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        &[],
    ).await?;

    // Sizes table
    client.execute(
        "CREATE TABLE IF NOT EXISTS sizes (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR NOT NULL UNIQUE,
            length_inches DOUBLE PRECISION,
            ring_gauge INTEGER,
            description TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        &[],
    ).await?;

    // Origins table
    client.execute(
        "CREATE TABLE IF NOT EXISTS origins (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR NOT NULL UNIQUE,
            country VARCHAR NOT NULL,
            region VARCHAR,
            description TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        &[],
    ).await?;

    // Strengths table
    client.execute(
        "CREATE TABLE IF NOT EXISTS strengths (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR NOT NULL UNIQUE,
            level INTEGER NOT NULL,
            description TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        &[],
    ).await?;

    // Ring Gauges table
    client.execute(
        "CREATE TABLE IF NOT EXISTS ring_gauges (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            gauge INTEGER NOT NULL UNIQUE,
            description TEXT,
            common_names VARCHAR[],
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        &[],
    ).await?;

    // Insert default strength values if the table is empty
    let strength_count: i64 = client
        .query_one("SELECT COUNT(*) FROM strengths", &[])
        .await?
        .get(0);
    
    if strength_count == 0 {
        client.execute(
            "INSERT INTO strengths (name, level, description) VALUES
             ('Mild', 1, 'Light and smooth, perfect for beginners'),
             ('Medium-Mild', 2, 'Slightly more body than mild, still approachable'),
             ('Medium', 3, 'Balanced strength with good complexity'),
             ('Medium-Full', 4, 'Strong flavor with substantial body'),
             ('Full', 5, 'Bold and intense, for experienced smokers')",
            &[],
        ).await?;
    }

    // Insert common ring gauges if the table is empty
    let ring_gauge_count: i64 = client
        .query_one("SELECT COUNT(*) FROM ring_gauges", &[])
        .await?
        .get(0);
    
    if ring_gauge_count == 0 {
        client.execute(
            "INSERT INTO ring_gauges (gauge, description, common_names) VALUES
             (38, 'Very thin gauge, quick smoke', ARRAY['Lancero thin', 'Panetela']),
             (42, 'Classic thin gauge', ARRAY['Corona', 'Petit Corona']),
             (44, 'Standard corona size', ARRAY['Corona', 'Lonsdale']),
             (46, 'Popular medium gauge', ARRAY['Corona Gorda', 'Petit Robusto']),
             (48, 'Medium-thick gauge', ARRAY['Robusto thin']),
             (50, 'Classic robusto gauge', ARRAY['Robusto', 'Rothschild']),
             (52, 'Thick robusto gauge', ARRAY['Robusto Extra', 'Toro thin']),
             (54, 'Toro gauge', ARRAY['Toro', 'Gordo']),
             (56, 'Churchill gauge', ARRAY['Churchill', 'Double Corona']),
             (58, 'Thick churchill', ARRAY['Churchill Extra']),
             (60, 'Very thick gauge', ARRAY['Gordo', 'Double Toro'])",
            &[],
        ).await?;
    }

    // Insert common brands if the table is empty
    let brand_count: i64 = client
        .query_one("SELECT COUNT(*) FROM brands", &[])
        .await?
        .get(0);
    
    if brand_count == 0 {
        client.execute(
            "INSERT INTO brands (name, description, country) VALUES
             ('Arturo Fuente', 'Premium Dominican cigars, known for OpusX and Hemingway lines', 'Dominican Republic'),
             ('Davidoff', 'Luxury Swiss brand with premium tobacco', 'Switzerland'),
             ('Padron', 'Family-owned Nicaraguan brand known for quality and consistency', 'Nicaragua'),
             ('Cohiba', 'Iconic Cuban brand, flagship of Habanos', 'Cuba'),
             ('Montecristo', 'One of the most recognized Cuban brands worldwide', 'Cuba'),
             ('Romeo y Julieta', 'Classic Cuban brand with wide variety', 'Cuba'),
             ('Partagas', 'Historic Cuban brand known for full-bodied cigars', 'Cuba'),
             ('Hoyo de Monterrey', 'Cuban brand known for mild to medium strength', 'Cuba'),
             ('Oliva', 'Nicaraguan family business with consistent quality', 'Nicaragua'),
             ('My Father', 'Premium Nicaraguan brand by Jose ''Pepin'' Garcia', 'Nicaragua'),
             ('Drew Estate', 'Innovative American brand, makers of Liga Privada and Acid', 'United States'),
             ('Rocky Patel', 'Popular brand with wide range of blends', 'Honduras'),
             ('Ashton', 'Premium brand with Dominican and Nicaraguan lines', 'United States'),
             ('Alec Bradley', 'Honduran brand known for Prensado and Black Market', 'Honduras'),
             ('La Flor Dominicana', 'Dominican brand known for powerful cigars', 'Dominican Republic'),
             ('Perdomo', 'Nicaraguan brand with extensive aging program', 'Nicaragua'),
             ('Tatuaje', 'Boutique brand known for Nicaraguan puros', 'Nicaragua'),
             ('Liga Privada', 'Premium line from Drew Estate', 'United States'),
             ('Punch', 'Cuban brand known for robust flavors', 'Cuba'),
             ('H. Upmann', 'Historic Cuban brand dating to 1844', 'Cuba')",
            &[],
        ).await?;
    }

    // Insert common origins if the table is empty
    let origin_count: i64 = client
        .query_one("SELECT COUNT(*) FROM origins", &[])
        .await?
        .get(0);
    
    if origin_count == 0 {
        client.execute(
            "INSERT INTO origins (name, country, region, description) VALUES
             ('Cuba', 'Cuba', NULL, 'Historic birthplace of premium cigars, known for rich flavor profiles'),
             ('Dominican Republic', 'Dominican Republic', NULL, 'World''s largest cigar producer, known for smooth, mild to medium cigars'),
             ('Nicaragua', 'Nicaragua', NULL, 'Produces full-bodied, peppery cigars with bold flavors'),
             ('Honduras', 'Honduras', NULL, 'Known for robust, flavorful cigars with Cuban-seed tobacco'),
             ('Mexico', 'Mexico', NULL, 'Produces rich, earthy cigars with quality wrapper tobacco'),
             ('United States', 'United States', NULL, 'Home to premium brands and innovative blends'),
             ('Ecuador', 'Ecuador', NULL, 'Famous for high-quality Connecticut Shade wrapper tobacco'),
             ('Brazil', 'Brazil', NULL, 'Known for dark, sweet maduro wrapper leaves'),
             ('Peru', 'Peru', NULL, 'Emerging origin with quality tobacco production'),
             ('Costa Rica', 'Costa Rica', NULL, 'Produces mild, smooth cigars with balanced flavor'),
             ('Panama', 'Panama', NULL, 'Small production of premium boutique cigars'),
             ('Colombia', 'Colombia', NULL, 'Growing reputation for quality tobacco'),
             ('Philippines', 'Philippines', NULL, 'Historic cigar production, value-priced offerings'),
             ('Indonesia', 'Indonesia', NULL, 'Known for Sumatra wrapper tobacco')",
            &[],
        ).await?;
    }

    // Insert common sizes if the table is empty
    let size_count: i64 = client
        .query_one("SELECT COUNT(*) FROM sizes", &[])
        .await?
        .get(0);
    
    if size_count == 0 {
        client.execute(
            "INSERT INTO sizes (name, length_inches, ring_gauge, description) VALUES
             ('Petit Corona', 4.5, 42, 'Small classic size, 30-40 minute smoke'),
             ('Corona', 5.5, 42, 'Traditional Cuban size, balanced proportions'),
             ('Corona Gorda', 5.625, 46, 'Larger corona with more body'),
             ('Petit Robusto', 4.0, 50, 'Short and thick, concentrated flavor'),
             ('Robusto', 5.0, 50, 'Most popular size, 45-60 minute smoke'),
             ('Robusto Extra', 5.5, 50, 'Longer robusto for extended enjoyment'),
             ('Toro', 6.0, 50, 'Popular modern size, well-balanced'),
             ('Gordo', 6.0, 60, 'Large ring gauge, cooler smoke'),
             ('Churchill', 7.0, 47, 'Named after Winston Churchill, elegant size'),
             ('Double Corona', 7.5, 50, 'Large premium size, 90+ minute smoke'),
             ('Lancero', 7.5, 38, 'Long and thin, concentrated flavors'),
             ('Panetela', 6.0, 34, 'Slim and elegant, quick smoke'),
             ('Lonsdale', 6.5, 42, 'Classic thin vitola, refined smoke'),
             ('Torpedo', 6.125, 52, 'Tapered head, concentrated flavors'),
             ('Belicoso', 5.0, 52, 'Short pyramid shape with tapered head'),
             ('Perfecto', 5.0, 48, 'Tapered at both ends, unique experience'),
             ('Presidente', 8.0, 50, 'Extra-long premium size'),
             ('Rothschild', 4.5, 50, 'Short robusto, rich and quick'),
             ('Corona Extra', 5.5, 46, 'Medium size with good balance'),
             ('Gigante', 9.0, 52, 'Exceptionally large, 2+ hour smoke')",
            &[],
        ).await?;
    }

    // NOW create cigars table (using foreign keys to organizer tables created above)
    client.execute(
        "CREATE TABLE IF NOT EXISTS cigars (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            humidor_id UUID REFERENCES humidors(id) ON DELETE SET NULL,
            brand_id UUID REFERENCES brands(id) ON DELETE SET NULL,
            name VARCHAR NOT NULL,
            size_id UUID REFERENCES sizes(id) ON DELETE SET NULL,
            strength_id UUID REFERENCES strengths(id) ON DELETE SET NULL,
            origin_id UUID REFERENCES origins(id) ON DELETE SET NULL,
            wrapper VARCHAR,
            binder VARCHAR,
            filler VARCHAR,
            price DOUBLE PRECISION,
            purchase_date TIMESTAMPTZ,
            notes TEXT,
            quantity INTEGER NOT NULL DEFAULT 1,
            ring_gauge_id UUID REFERENCES ring_gauges(id) ON DELETE SET NULL,
            length DOUBLE PRECISION,
            image_url TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        &[],
    ).await?;
    
    // Create indexes for better query performance
    client.execute("CREATE INDEX IF NOT EXISTS idx_cigars_brand_id ON cigars(brand_id)", &[]).await?;
    client.execute("CREATE INDEX IF NOT EXISTS idx_cigars_size_id ON cigars(size_id)", &[]).await?;
    client.execute("CREATE INDEX IF NOT EXISTS idx_cigars_origin_id ON cigars(origin_id)", &[]).await?;
    client.execute("CREATE INDEX IF NOT EXISTS idx_cigars_strength_id ON cigars(strength_id)", &[]).await?;
    client.execute("CREATE INDEX IF NOT EXISTS idx_cigars_ring_gauge_id ON cigars(ring_gauge_id)", &[]).await?;

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

    // Combine all API routes
    let api = get_cigars
        .or(create_cigar)
        .or(get_cigar)
        .or(update_cigar)
        .or(delete_cigar)
        .or(scrape_cigar)
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
        .or(get_humidor_cigars)  // Must come before get_humidor (more specific route)
        .or(create_humidor)
        .or(update_humidor)
        .or(delete_humidor)
        .or(get_humidor);  // Less specific, should be last

    // Root route
    let root = warp::path::end()
        .and(warp::get())
        .and_then(serve_index);

    // Setup route
    let setup = warp::path("setup.html")
        .and(warp::get())
        .and_then(serve_setup);

    // Login route
    let login = warp::path("login.html")
        .and(warp::get())
        .and_then(serve_login);

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);

    let routes = root
        .or(setup)
        .or(login)
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

async fn serve_login() -> Result<impl Reply, warp::Rejection> {
    match tokio::fs::read_to_string("static/login.html").await {
        Ok(content) => Ok(warp::reply::html(content).into_response()),
        Err(_) => Ok(warp::reply::with_status(
            warp::reply::html("<h1>Login Not Found</h1>".to_string()),
            warp::http::StatusCode::NOT_FOUND,
        ).into_response()),
    }
}
