use crate::models::{User, CreateUserRequest, LoginRequest, LoginResponse, UserResponse, SetupStatusResponse, CreateHumidorRequest, Humidor, SetupRequest};
use crate::DbPool;
use warp::Reply;
use std::collections::HashMap;
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;

// Authentication and JWT utilities
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use bcrypt::{hash, verify, DEFAULT_COST};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub username: String,
    pub exp: usize, // expiration time
}

const JWT_SECRET: &str = "your-secret-key"; // In production, use environment variable

// Setup endpoints
pub async fn get_setup_status(db: DbPool) -> Result<impl Reply, warp::Rejection> {
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
    db: DbPool,
) -> Result<impl Reply, warp::Rejection> {
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
            ).into_response());
        }
    }
    
    // Hash password
    let password_hash = match hash(&setup_req.user.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("Password hashing error: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "error": "Failed to process password"
                })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ).into_response());
        }
    };
    
    let user_id = Uuid::new_v4();
    let now = Utc::now();
    
    let query = "
        INSERT INTO users (id, username, email, full_name, password_hash, is_admin, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, username, email, full_name, is_admin, is_active, created_at, updated_at
    ";
    
    match db.query_one(
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
    ).await {
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
                    ).into_response());
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
                INSERT INTO humidors (id, user_id, name, description, capacity, target_humidity, location, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ";
            
            if let Err(e) = db.execute(
                humidor_query,
                &[
                    &humidor_id,
                    &user_id,
                    &setup_req.humidor.name,
                    &setup_req.humidor.description,
                    &setup_req.humidor.capacity,
                    &setup_req.humidor.target_humidity,
                    &setup_req.humidor.location,
                    &now,
                    &now,
                ],
            ).await {
                eprintln!("Failed to create humidor: {}", e);
                return Ok(warp::reply::with_status(
                    warp::reply::json(&json!({
                        "error": "Failed to create humidor"
                    })),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                ).into_response());
            }
            
            let response = LoginResponse {
                user: user_response,
                token,
            };
            
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
                ).into_response())
            } else {
                Ok(warp::reply::with_status(
                    warp::reply::json(&json!({
                        "error": "Failed to create user"
                    })),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                ).into_response())
            }
        }
    }
}

// Regular authentication endpoints
pub async fn login_user(
    login_req: LoginRequest,
    db: DbPool,
) -> Result<impl Reply, warp::Rejection> {
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
                ).into_response());
            }
            
            match verify(&login_req.password, &password_hash) {
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
                            ).into_response());
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
                Ok(false) => {
                    Ok(warp::reply::with_status(
                        warp::reply::json(&json!({
                            "error": "Invalid username or password"
                        })),
                        warp::http::StatusCode::UNAUTHORIZED,
                    ).into_response())
                }
                Err(e) => {
                    eprintln!("Password verification error: {}", e);
                    Ok(warp::reply::with_status(
                        warp::reply::json(&json!({
                            "error": "Authentication failed"
                        })),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ).into_response())
                }
            }
        }
        Ok(None) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "error": "Invalid username or password"
                })),
                warp::http::StatusCode::UNAUTHORIZED,
            ).into_response())
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "error": "Authentication failed"
                })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ).into_response())
        }
    }
}

// Humidor endpoints for setup
pub async fn create_humidor_for_setup(
    humidor_req: CreateHumidorRequest,
    user_id: String, // This would come from JWT auth middleware
    db: DbPool,
) -> Result<impl Reply, warp::Rejection> {
    let humidor_id = Uuid::new_v4();
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "error": "Invalid user ID"
                })),
                warp::http::StatusCode::BAD_REQUEST,
            ).into_response());
        }
    };
    let now = Utc::now();
    
    let query = "
        INSERT INTO humidors (id, user_id, name, description, capacity, target_humidity, location, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, user_id, name, description, capacity, target_humidity, location, created_at, updated_at
    ";
    
    match db.query_one(
        query,
        &[
            &humidor_id,
            &user_uuid,
            &humidor_req.name,
            &humidor_req.description,
            &humidor_req.capacity,
            &humidor_req.target_humidity,
            &humidor_req.location,
            &now,
            &now,
        ],
    ).await {
        Ok(row) => {
            let humidor = Humidor {
                id: row.get("id"),
                user_id: row.get("user_id"),
                name: row.get("name"),
                description: row.get("description"),
                capacity: row.get("capacity"),
                target_humidity: row.get("target_humidity"),
                location: row.get("location"),
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
            ).into_response())
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
    let key = EncodingKey::from_secret(JWT_SECRET.as_ref());
    
    encode(&header, &claims, &key)
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let key = DecodingKey::from_secret(JWT_SECRET.as_ref());
    let validation = Validation::new(Algorithm::HS256);
    
    decode::<Claims>(token, &key, &validation).map(|data| data.claims)
}