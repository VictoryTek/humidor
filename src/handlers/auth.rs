use crate::errors::AppError;
use crate::models::{
    CreateHumidorRequest, Humidor, LoginRequest, LoginResponse, SetupRequest, SetupStatusResponse,
    UserResponse,
};
use crate::DbPool;
use chrono::Utc;
use serde_json::json;
use std::env;
use uuid::Uuid;
use warp::Reply;

// Authentication and JWT utilities
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

// Async-safe bcrypt operations using tokio::task::spawn_blocking
async fn hash_password(password: String) -> Result<String, bcrypt::BcryptError> {
    tokio::task::spawn_blocking(move || hash(&password, DEFAULT_COST))
        .await
        .map_err(|e| {
            eprintln!("Task join error during password hashing: {}", e);
            bcrypt::BcryptError::InvalidCost(DEFAULT_COST.to_string())
        })?
}

async fn verify_password(password: String, hash_str: String) -> Result<bool, bcrypt::BcryptError> {
    tokio::task::spawn_blocking(move || verify(&password, &hash_str))
        .await
        .map_err(|e| {
            eprintln!("Task join error during password verification: {}", e);
            bcrypt::BcryptError::InvalidHash("".to_string())
        })?
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub username: String,
    pub exp: usize, // expiration time
}

/// Get JWT secret from environment variable
/// This function will panic at startup if JWT_SECRET is not set, which is intentional
/// to prevent running with an insecure default
fn jwt_secret() -> String {
    env::var("JWT_SECRET").expect(
        "JWT_SECRET environment variable must be set. Generate one with: openssl rand -base64 32",
    )
}

// Setup endpoints
pub async fn get_setup_status(pool: DbPool) -> Result<impl Reply, warp::Rejection> {
    let db = pool.get().await.map_err(|e| {
        eprintln!("Failed to get database connection: {}", e);
        warp::reject::custom(AppError::DatabaseError("Database connection failed".to_string()))
    })?;

    let query = "SELECT COUNT(*) FROM users WHERE is_admin = true";

    match db.query_one(query, &[]).await {
        Ok(row) => {
            let admin_count: i64 = row.get(0);
            let needs_setup = admin_count == 0;

            let response = SetupStatusResponse {
                needs_setup,
                has_admin: !needs_setup,
            };

            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({
                "error": "Failed to check setup status"
            })))
        }
    }
}

pub async fn create_setup_user(
    setup_req: SetupRequest,
    pool: DbPool,
) -> Result<impl Reply, warp::Rejection> {
    let db = pool.get().await.map_err(|e| {
        eprintln!("Failed to get database connection: {}", e);
        warp::reject::custom(AppError::DatabaseError("Database connection failed".to_string()))
    })?;

    // Check if setup is still needed
    let check_query = "SELECT COUNT(*) FROM users WHERE is_admin = true";
    if let Ok(row) = db.query_one(check_query, &[]).await {
        let admin_count: i64 = row.get(0);
        if admin_count > 0 {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "error": "Setup has already been completed"
                })),
                warp::http::StatusCode::BAD_REQUEST,
            )
            .into_response());
        }
    }

    // Hash password (using spawn_blocking to avoid blocking async runtime)
    let password_hash = match hash_password(setup_req.user.password.clone()).await {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("Password hashing error: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "error": "Failed to process password"
                })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response());
        }
    };

    let user_id = Uuid::new_v4();
    let now = Utc::now();

    let query = "
        INSERT INTO users (id, username, email, full_name, password_hash, is_admin, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, username, email, full_name, is_admin, is_active, created_at, updated_at
    ";

    match db
        .query_one(
            query,
            &[
                &user_id,
                &setup_req.user.username,
                &setup_req.user.email,
                &setup_req.user.full_name,
                &password_hash,
                &true, // First user is admin
                &true, // Active by default
                &now,
                &now,
            ],
        )
        .await
    {
        Ok(row) => {
            // Generate JWT token
            let token = match generate_token(&user_id.to_string(), &setup_req.user.username) {
                Ok(token) => token,
                Err(e) => {
                    eprintln!("Token generation error: {}", e);
                    return Ok(warp::reply::with_status(
                        warp::reply::json(&json!({
                            "error": "Failed to generate authentication token"
                        })),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    )
                    .into_response());
                }
            };

            let user_response = UserResponse {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                full_name: row.get("full_name"),
                is_admin: row.get("is_admin"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };

            // Create the first humidor
            let humidor_id = Uuid::new_v4();
            let humidor_query = "
                INSERT INTO humidors (id, user_id, name, description, capacity, target_humidity, location, is_wishlist, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ";

            if let Err(e) = db
                .execute(
                    humidor_query,
                    &[
                        &humidor_id,
                        &user_id,
                        &setup_req.humidor.name,
                        &setup_req.humidor.description,
                        &setup_req.humidor.capacity,
                        &setup_req.humidor.target_humidity,
                        &setup_req.humidor.location,
                        &false, // is_wishlist
                        &now,
                        &now,
                    ],
                )
                .await
            {
                eprintln!("Failed to create humidor: {}", e);
                return Ok(warp::reply::with_status(
                    warp::reply::json(&json!({
                        "error": "Failed to create humidor"
                    })),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                )
                .into_response());
            }

            let response = json!({
                "user": user_response,
                "token": token,
                "humidor_id": humidor_id.to_string()
            });

            Ok(warp::reply::json(&response).into_response())
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            if e.to_string().contains("duplicate key") {
                Ok(warp::reply::with_status(
                    warp::reply::json(&json!({
                        "error": "Username or email already exists"
                    })),
                    warp::http::StatusCode::CONFLICT,
                )
                .into_response())
            } else {
                Ok(warp::reply::with_status(
                    warp::reply::json(&json!({
                        "error": "Failed to create user"
                    })),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                )
                .into_response())
            }
        }
    }
}

// Regular authentication endpoints
pub async fn login_user(
    login_req: LoginRequest,
    pool: DbPool,
) -> Result<impl Reply, warp::Rejection> {
    let db = pool.get().await.map_err(|e| {
        eprintln!("Failed to get database connection: {}", e);
        warp::reject::custom(AppError::DatabaseError("Database connection failed".to_string()))
    })?;

    let query = "
        SELECT id, username, email, full_name, password_hash, is_admin, is_active, created_at, updated_at
        FROM users 
        WHERE username = $1 OR email = $1
    ";

    match db.query_opt(query, &[&login_req.username]).await {
        Ok(Some(row)) => {
            let password_hash: String = row.get("password_hash");
            let is_active: bool = row.get("is_active");

            if !is_active {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&json!({
                        "error": "Account is disabled"
                    })),
                    warp::http::StatusCode::UNAUTHORIZED,
                )
                .into_response());
            }

            // Verify password (using spawn_blocking to avoid blocking async runtime)
            match verify_password(login_req.password.clone(), password_hash).await {
                Ok(true) => {
                    let user_id: Uuid = row.get("id");
                    let username: String = row.get("username");

                    let token = match generate_token(&user_id.to_string(), &username) {
                        Ok(token) => token,
                        Err(e) => {
                            eprintln!("Token generation error: {}", e);
                            return Ok(warp::reply::with_status(
                                warp::reply::json(&json!({
                                    "error": "Authentication failed"
                                })),
                                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                            )
                            .into_response());
                        }
                    };

                    let user_response = UserResponse {
                        id: row.get("id"),
                        username: row.get("username"),
                        email: row.get("email"),
                        full_name: row.get("full_name"),
                        is_admin: row.get("is_admin"),
                        is_active: row.get("is_active"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                    };

                    let response = LoginResponse {
                        user: user_response,
                        token,
                    };

                    Ok(warp::reply::json(&response).into_response())
                }
                Ok(false) => Ok(warp::reply::with_status(
                    warp::reply::json(&json!({
                        "error": "Invalid username or password"
                    })),
                    warp::http::StatusCode::UNAUTHORIZED,
                )
                .into_response()),
                Err(e) => {
                    eprintln!("Password verification error: {}", e);
                    Ok(warp::reply::with_status(
                        warp::reply::json(&json!({
                            "error": "Authentication failed"
                        })),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    )
                    .into_response())
                }
            }
        }
        Ok(None) => Ok(warp::reply::with_status(
            warp::reply::json(&json!({
                "error": "Invalid username or password"
            })),
            warp::http::StatusCode::UNAUTHORIZED,
        )
        .into_response()),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "error": "Authentication failed"
                })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response())
        }
    }
}

// Humidor endpoints for setup
#[allow(dead_code)]
pub async fn create_humidor_for_setup(
    humidor_req: CreateHumidorRequest,
    user_id: String, // This would come from JWT auth middleware
    pool: DbPool,
) -> Result<impl Reply, warp::Rejection> {
    let db = pool.get().await.map_err(|e| {
        eprintln!("Failed to get database connection: {}", e);
        warp::reject::custom(AppError::DatabaseError("Database connection failed".to_string()))
    })?;

    let humidor_id = Uuid::new_v4();
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "error": "Invalid user ID"
                })),
                warp::http::StatusCode::BAD_REQUEST,
            )
            .into_response());
        }
    };
    let now = Utc::now();

    let query = "
        INSERT INTO humidors (id, user_id, name, description, capacity, target_humidity, location, is_wishlist, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, user_id, name, description, capacity, target_humidity, location, is_wishlist, created_at, updated_at
    ";

    match db
        .query_one(
            query,
            &[
                &humidor_id,
                &user_uuid,
                &humidor_req.name,
                &humidor_req.description,
                &humidor_req.capacity,
                &humidor_req.target_humidity,
                &humidor_req.location,
                &false, // is_wishlist
                &now,
                &now,
            ],
        )
        .await
    {
        Ok(row) => {
            let humidor = Humidor {
                id: row.get("id"),
                user_id: row.get("user_id"),
                name: row.get("name"),
                description: row.get("description"),
                capacity: row.get("capacity"),
                target_humidity: row.get("target_humidity"),
                location: row.get("location"),
                is_wishlist: row.get("is_wishlist"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };

            Ok(warp::reply::json(&humidor).into_response())
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "error": "Failed to create humidor"
                })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response())
        }
    }
}

// JWT token utilities
fn generate_token(user_id: &str, username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_owned(),
        username: username.to_owned(),
        exp: expiration,
    };

    let header = Header::new(Algorithm::HS256);
    let secret = jwt_secret();
    let key = EncodingKey::from_secret(secret.as_bytes());

    encode(&header, &claims, &key)
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = jwt_secret();
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);

    decode::<Claims>(token, &key, &validation).map(|data| data.claims)
}

// User profile management
pub async fn get_current_user(
    auth: crate::middleware::AuthContext,
    pool: DbPool,
) -> Result<impl Reply, warp::Rejection> {
    let db = pool.get().await.map_err(|e| {
        eprintln!("Failed to get database connection: {}", e);
        warp::reject::custom(AppError::DatabaseError("Database connection failed".to_string()))
    })?;

    match db.query_one(
        "SELECT id, username, email, full_name, is_admin, is_active, created_at, updated_at FROM users WHERE id = $1",
        &[&auth.user_id]
    ).await {
        Ok(row) => {
            let user = UserResponse {
                id: row.get(0),
                username: row.get(1),
                email: row.get(2),
                full_name: row.get(3),
                is_admin: row.get(4),
                is_active: row.get(5),
                created_at: row.get(6),
                updated_at: row.get(7),
            };
            Ok(warp::reply::json(&user))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to fetch user"})))
        }
    }
}

pub async fn update_current_user(
    update_req: crate::models::UpdateUserRequest,
    auth: crate::middleware::AuthContext,
    pool: DbPool,
) -> Result<impl Reply, warp::Rejection> {
    let db = pool.get().await.map_err(|e| {
        eprintln!("Failed to get database connection: {}", e);
        warp::reject::custom(AppError::DatabaseError("Database connection failed".to_string()))
    })?;

    let mut updates = Vec::new();
    let mut values: Vec<String> = Vec::new();
    let mut param_index = 1;

    if let Some(username) = &update_req.username {
        updates.push(format!("username = ${}", param_index));
        values.push(username.clone());
        param_index += 1;
    }

    if let Some(email) = &update_req.email {
        updates.push(format!("email = ${}", param_index));
        values.push(email.clone());
        param_index += 1;
    }

    if let Some(full_name) = &update_req.full_name {
        updates.push(format!("full_name = ${}", param_index));
        values.push(full_name.clone());
        param_index += 1;
    }

    if updates.is_empty() {
        return Ok(warp::reply::json(
            &json!({"message": "No updates provided"}),
        ));
    }

    updates.push(format!("updated_at = ${}", param_index));
    let now = Utc::now();

    let query = format!(
        "UPDATE users SET {} WHERE id = ${} RETURNING id, username, email, full_name, is_admin, is_active, created_at, updated_at",
        updates.join(", "),
        param_index + 1
    );

    // Build parameters
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
    for value in &values {
        params.push(value);
    }
    params.push(&now);
    params.push(&auth.user_id);

    match db.query_one(&query, &params).await {
        Ok(row) => {
            let user = UserResponse {
                id: row.get(0),
                username: row.get(1),
                email: row.get(2),
                full_name: row.get(3),
                is_admin: row.get(4),
                is_active: row.get(5),
                created_at: row.get(6),
                updated_at: row.get(7),
            };
            Ok(warp::reply::json(&user))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(
                &json!({"error": "Failed to update user"}),
            ))
        }
    }
}

pub async fn change_password(
    password_req: crate::models::ChangePasswordRequest,
    auth: crate::middleware::AuthContext,
    pool: DbPool,
) -> Result<impl Reply, warp::Rejection> {
    let db = pool.get().await.map_err(|e| {
        eprintln!("Failed to get database connection: {}", e);
        warp::reject::custom(AppError::DatabaseError("Database connection failed".to_string()))
    })?;

    // Get current password hash
    match db
        .query_one(
            "SELECT password_hash FROM users WHERE id = $1",
            &[&auth.user_id],
        )
        .await
    {
        Ok(row) => {
            let current_hash: String = row.get(0);

            // Verify current password (using spawn_blocking to avoid blocking async runtime)
            match verify_password(password_req.current_password.clone(), current_hash).await {
                Ok(valid) => {
                    if !valid {
                        return Ok(warp::reply::json(
                            &json!({"error": "Current password is incorrect"}),
                        ));
                    }
                }
                Err(e) => {
                    eprintln!("Password verification error: {}", e);
                    return Ok(warp::reply::json(
                        &json!({"error": "Password verification failed"}),
                    ));
                }
            }

            // Hash new password (using spawn_blocking to avoid blocking async runtime)
            let new_hash = match hash_password(password_req.new_password.clone()).await {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("Password hashing error: {}", e);
                    return Ok(warp::reply::json(
                        &json!({"error": "Failed to hash new password"}),
                    ));
                }
            };

            // Update password
            let now = Utc::now();
            match db
                .execute(
                    "UPDATE users SET password_hash = $1, updated_at = $2 WHERE id = $3",
                    &[&new_hash, &now, &auth.user_id],
                )
                .await
            {
                Ok(_) => Ok(warp::reply::json(
                    &json!({"message": "Password updated successfully"}),
                )),
                Err(e) => {
                    eprintln!("Database error: {}", e);
                    Ok(warp::reply::json(
                        &json!({"error": "Failed to update password"}),
                    ))
                }
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::json(&json!({"error": "Failed to fetch user"})))
        }
    }
}
