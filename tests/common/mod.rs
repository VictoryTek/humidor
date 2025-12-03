use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use refinery::embed_migrations;
use std::env;
use tokio_postgres::NoTls;
use uuid::Uuid;

embed_migrations!("migrations");

pub struct TestContext {
    pub pool: Pool,
}

/// Set up a test database with migrations
/// Uses the existing Docker Compose PostgreSQL instance
pub async fn setup_test_db() -> TestContext {
    let database_url = env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://humidor_user:humidor_pass@localhost:5432/humidor_db".to_string()
    });

    // Create connection pool
    let mut config = Config::new();
    config.url = Some(database_url.clone());
    config.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    let pool = config
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("Failed to create pool");

    // Run migrations first to ensure all tables exist
    // Use run_async which will skip already-applied migrations
    {
        let mut client = pool.get().await.expect("Failed to get client");
        // Migrations will be skipped if already applied - this is safe
        let _ = migrations::runner().run_async(&mut **client).await;
    } // Release client

    // Manually create wish_list table if it doesn't exist (V8 migration not embedded yet)
    {
        let client = pool
            .get()
            .await
            .expect("Failed to get client for wish_list setup");
        let _ = client
            .batch_execute(
                "
            CREATE TABLE IF NOT EXISTS wish_list (
                id UUID PRIMARY KEY,
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                cigar_id UUID NOT NULL REFERENCES cigars(id) ON DELETE CASCADE,
                notes TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT NOW(),
                UNIQUE(user_id, cigar_id)
            );
            CREATE INDEX IF NOT EXISTS idx_wish_list_user_id ON wish_list(user_id);
            CREATE INDEX IF NOT EXISTS idx_wish_list_cigar_id ON wish_list(cigar_id);
            CREATE INDEX IF NOT EXISTS idx_wish_list_created ON wish_list(created_at DESC);
        ",
            )
            .await;
    }

    // Ensure retail_link column exists (V10 migration)
    {
        let client = pool
            .get()
            .await
            .expect("Failed to get client for retail_link setup");
        let _ = client
            .batch_execute(
                "
            ALTER TABLE cigars ADD COLUMN IF NOT EXISTS retail_link TEXT;
            CREATE INDEX IF NOT EXISTS idx_cigars_retail_link ON cigars(retail_link) WHERE retail_link IS NOT NULL;
        ",
            )
            .await;
    }

    // Ensure humidor_shares table exists (V12 migration)
    {
        let client = pool
            .get()
            .await
            .expect("Failed to get client for humidor_shares setup");
        let _ = client
            .batch_execute(
                "
            CREATE TABLE IF NOT EXISTS humidor_shares (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                humidor_id UUID NOT NULL REFERENCES humidors(id) ON DELETE CASCADE,
                shared_with_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                shared_by_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                permission_level VARCHAR(20) NOT NULL CHECK (permission_level IN ('view', 'edit', 'full')),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                CONSTRAINT unique_humidor_share UNIQUE (humidor_id, shared_with_user_id)
            );
            CREATE INDEX IF NOT EXISTS idx_humidor_shares_humidor_id ON humidor_shares(humidor_id);
            CREATE INDEX IF NOT EXISTS idx_humidor_shares_shared_with_user_id ON humidor_shares(shared_with_user_id);
            CREATE INDEX IF NOT EXISTS idx_humidor_shares_shared_by_user_id ON humidor_shares(shared_by_user_id);
        ",
            )
            .await;
    }

    // Now clean up any existing test data with a fresh client
    {
        let client = pool.get().await.expect("Failed to get client for cleanup");
        // Use DELETE to clean up test data (order matters due to foreign keys)
        let _ = client.execute("DELETE FROM humidor_shares", &[]).await;
        let _ = client.execute("DELETE FROM wish_list", &[]).await;
        let _ = client.execute("DELETE FROM favorites", &[]).await;
        let _ = client.execute("DELETE FROM cigars", &[]).await;
        let _ = client.execute("DELETE FROM humidors", &[]).await;
        let _ = client.execute("DELETE FROM users", &[]).await;
    }

    TestContext { pool }
}

/// Create a test user and return their ID and unique username
/// Generates unique username by appending UUID to avoid conflicts
pub async fn create_test_user(
    pool: &Pool,
    username: &str,
    password: &str,
    is_admin: bool,
) -> Result<(Uuid, String), Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    // Make username unique by appending UUID
    let unique_username = format!("{}_{}", username, Uuid::new_v4());

    // Hash password
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;

    let row = client
        .query_one(
            "INSERT INTO users (id, username, email, full_name, password_hash, is_admin, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW()) 
             RETURNING id",
            &[
                &Uuid::new_v4(),
                &unique_username,
                &format!("{}@test.com", unique_username),
                &unique_username, // Use username as full_name for tests
                &password_hash,
                &is_admin,
            ],
        )
        .await?;

    Ok((row.get(0), unique_username))
}

/// Create a test user and get JWT token
#[allow(dead_code)]
pub async fn create_user_and_login(
    pool: &Pool,
    username: &str,
    password: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let (user_id, unique_username) = create_test_user(pool, username, password, false).await?;

    // Create a simple JWT token for testing
    // In real tests, you'd call the actual login endpoint
    let token = create_test_jwt(user_id, &unique_username)?;
    Ok(token)
}

/// Create a JWT token for testing
#[allow(dead_code)]
fn create_test_jwt(user_id: Uuid, username: &str) -> Result<String, Box<dyn std::error::Error>> {
    use jsonwebtoken::{EncodingKey, Header, encode};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        user_id: String,
        username: String,
        exp: usize,
        iat: usize,
    }

    let secret =
        env::var("JWT_SECRET").unwrap_or_else(|_| "test_secret_key_for_testing".to_string());
    let now = chrono::Utc::now().timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        user_id: user_id.to_string(),
        username: username.to_string(),
        exp: now + 3600, // 1 hour
        iat: now,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

/// Create a test humidor for a user
#[allow(dead_code)]
pub async fn create_test_humidor(
    pool: &Pool,
    user_id: Uuid,
    name: &str,
) -> Result<Uuid, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let row = client
        .query_one(
            "INSERT INTO humidors (id, name, user_id, created_at, updated_at) 
             VALUES ($1, $2, $3, NOW(), NOW()) 
             RETURNING id",
            &[&Uuid::new_v4(), &name, &user_id],
        )
        .await?;

    Ok(row.get(0))
}

/// Create a test cigar
/// If humidor_id is None, creates a test user and humidor to ensure proper ownership
#[allow(dead_code)]
pub async fn create_test_cigar(
    pool: &Pool,
    name: &str,
    quantity: i32,
    humidor_id: Option<Uuid>,
) -> Result<Uuid, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    // Ensure we have a humidor_id (create user + humidor if needed)
    let actual_humidor_id = match humidor_id {
        Some(id) => id,
        None => {
            // Create a test user
            let (user_id, _) = create_test_user(pool, "cigar_test_user", "password", false).await?;
            // Create a humidor for this user
            create_test_humidor(pool, user_id, "Test Humidor").await?
        }
    };

    let cigar_id = Uuid::new_v4();

    // Don't explicitly list retail_link - let the database handle the default (NULL)
    client
        .execute(
            "INSERT INTO cigars (id, name, quantity, humidor_id, is_active, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, true, NOW(), NOW())",
            &[&cigar_id, &name, &quantity, &actual_humidor_id],
        )
        .await?;

    Ok(cigar_id)
}

/// Clean up database (delete all test data)
#[allow(dead_code)]
pub async fn cleanup_db(pool: &Pool) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    // Delete in order to respect foreign key constraints
    client.execute("DELETE FROM humidor_shares", &[]).await?;
    client.execute("DELETE FROM wish_list", &[]).await?;
    client.execute("DELETE FROM favorites", &[]).await?;
    client.execute("DELETE FROM cigars", &[]).await?;
    client.execute("DELETE FROM humidors", &[]).await?;
    client.execute("DELETE FROM users", &[]).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_test_db() {
        let ctx = setup_test_db().await;

        // Verify we can connect
        let client = ctx.pool.get().await.expect("Failed to get client");
        let row = client
            .query_one("SELECT 1 as num", &[])
            .await
            .expect("Failed to query");
        let num: i32 = row.get(0);
        assert_eq!(num, 1);
    }

    #[tokio::test]
    async fn test_create_test_user() {
        let ctx = setup_test_db().await;

        let (user_id, actual_username) =
            create_test_user(&ctx.pool, "testuser", "password123", false)
                .await
                .expect("Failed to create user");

        // Verify user was created
        let client = ctx.pool.get().await.unwrap();
        let row = client
            .query_one("SELECT username FROM users WHERE id = $1", &[&user_id])
            .await
            .unwrap();

        let username: String = row.get(0);
        assert_eq!(username, actual_username);
        assert!(username.starts_with("testuser_"));
    }
}
