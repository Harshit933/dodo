use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use time::OffsetDateTime;
use tracing::error;
use std::env;

use crate::models::user::{User, CreateUser, LoginUser, AuthResponse, RegisterResponse};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // user id
    exp: i64,    // expiration time
}

pub async fn register_user(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<RegisterResponse>, (StatusCode, String)> {
    tracing::info!("Starting registration for user: {}", payload.email);
    
    // Check if user already exists
    tracing::info!("Checking if user already exists");
    let existing_user = sqlx::query!(
        "SELECT id FROM users WHERE email = $1",
        payload.email
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error checking existing user: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    if existing_user.is_some() {
        tracing::error!("User already exists: {}", payload.email);
        return Err((StatusCode::CONFLICT, "User already exists".to_string()));
    }

    // Validate email format
    if !payload.email.contains('@') {
        error!("Invalid email format: {}", payload.email);
        return Err((StatusCode::BAD_REQUEST, "Invalid email format".to_string()));
    }

    // Validate password length
    if payload.password.len() < 8 {
        error!("Password too short");
        return Err((StatusCode::BAD_REQUEST, "Password must be at least 8 characters long".to_string()));
    }

    // Hash password
    tracing::info!("Hashing password");
    let password_hash = match hash(payload.password.as_bytes(), DEFAULT_COST) {
        Ok(hash) => {
            tracing::info!("Password hashed successfully");
            hash
        },
        Err(e) => {
            tracing::error!("Failed to hash password: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to hash password: {}", e)));
        }
    };

    // Create user
    tracing::info!("Creating new user in database");
    let user = match sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, password_hash, name)
        VALUES ($1, $2, $3)
        RETURNING id, email, password_hash, name, created_at, updated_at
        "#,
        payload.email,
        password_hash,
        payload.name
    )
    .fetch_one(&pool)
    .await {
        Ok(user) => {
            tracing::info!("User created successfully: {}", user.email);
            user
        },
        Err(e) => {
            tracing::error!("Failed to create user: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create user: {}", e)));
        }
    };

    tracing::info!("Registration completed successfully for user: {}", user.email);
    Ok(Json(RegisterResponse {
        message: "User registered successfully".to_string(),
        user,
    }))
}

pub async fn authenticate_user(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    tracing::info!("Starting authentication for user: {}", payload.email);
    
    // Find user
    tracing::info!("Querying database for user");
    let user = match sqlx::query_as!(
        User,
        r#"
        SELECT id, email, password_hash, name, created_at, updated_at
        FROM users
        WHERE email = $1
        "#,
        payload.email
    )
    .fetch_optional(&pool)
    .await {
        Ok(Some(user)) => {
            tracing::info!("User found in database");
            user
        },
        Ok(None) => {
            tracing::error!("User not found: {}", payload.email);
            return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
        },
        Err(e) => {
            tracing::error!("Database error during user lookup: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)));
        }
    };

    // Verify password
    tracing::info!("Verifying password");
    match verify(&payload.password, &user.password_hash) {
        Ok(true) => {
            tracing::info!("Password verified successfully");
        },
        Ok(false) => {
            tracing::error!("Invalid password for user: {}", payload.email);
            return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
        },
        Err(e) => {
            tracing::error!("Error verifying password: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to verify password: {}", e)));
        }
    }

    // Generate JWT
    tracing::info!("Generating JWT token");
    let token = match generate_token(&user.id) {
        Ok(token) => {
            tracing::info!("JWT token generated successfully");
            token
        },
        Err(e) => {
            tracing::error!("Failed to generate JWT token: {:?}", e);
            return Err(e);
        }
    };

    tracing::info!("Successfully authenticated user: {}", user.email);
    Ok(Json(AuthResponse { token, user }))
}

fn generate_token(user_id: &Uuid) -> Result<String, (StatusCode, String)> {
    let expiration = OffsetDateTime::now_utc().unix_timestamp() + 24 * 3600;

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
    };

    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
        tracing::error!("JWT_SECRET environment variable not set");
        "your-secret-key".to_string()
    });

    tracing::info!("Using JWT secret key length: {}", jwt_secret.len());

    match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes())
    ) {
        Ok(token) => {
            tracing::info!("Token generated successfully");
            Ok(token)
        },
        Err(e) => {
            tracing::error!("Failed to generate token: {:?}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate token: {}", e)))
        }
    }
} 