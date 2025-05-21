use axum::Router;
use axum::routing::{get, post};
use axum::middleware;
use axum::http::{HeaderValue, StatusCode};
use axum::extract::State;
use tokio::net::TcpListener;
use sqlx::postgres::PgPoolOptions;
use std::env;

use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod models;
mod handlers;

// Logging middleware
async fn logging_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> axum::http::Response<axum::body::Body> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    
    tracing::info!("{} {}", method, uri);
    
    let response = next.run(req).await;
    
    let status = response.status();
    if status.is_server_error() {
        tracing::error!("{} {} - {}", method, uri, status);
    } else {
        tracing::info!("{} {} - {}", method, uri, status);
    }
    
    response
}

// Health check handler
async fn health_check(
    State(pool): State<sqlx::PgPool>
) -> Result<String, (StatusCode, String)> {
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => Ok("Database connection OK".to_string()),
        Err(e) => {
            tracing::error!("Database health check failed: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database connection error: {}", e)))
        },
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing with more detailed logging
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("debug,tower_http=debug,axum=debug,sqlx=debug")))
        .with(tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .with_thread_names(true)
            .with_level(true)
            .with_ansi(true))
        .init();

    tracing::info!("Starting application...");

    // Load .env file
    dotenvy::dotenv().ok();
    
    // Set up database connection pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    tracing::info!("Connecting to database at: {}", database_url);
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    // Run migrations
    tracing::info!("Running database migrations");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true);

    // Create router with shared state
    let app = Router::new()
        // Health check endpoint
        .route("/health", get(health_check))
        // Auth endpoints
        .route("/v1/auth", post(handlers::auth::authenticate_user))
        .route("/v1/register", post(handlers::auth::register_user))
        
        // Transaction endpoints
        .route("/v1/users/{user_id}/transactions", post(handlers::transaction::create_transaction))
        .route("/v1/users/{user_id}/transactions", get(handlers::transaction::get_transactions))
        .route("/v1/users/{user_id}/balance", get(handlers::transaction::get_account_balance))
        .with_state(pool)
        // Add middleware layers
        .layer(middleware::from_fn(logging_middleware))
        .layer(cors)
        .layer(RequestBodyLimitLayer::new(1024 * 1024));

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    tracing::info!("Server running on http://127.0.0.1:8080");
    axum::serve(listener, app).await.unwrap();
}
