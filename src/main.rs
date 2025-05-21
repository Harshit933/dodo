/*

Requirements:
1. User registration
- Mostly with email, should store password by hashing.
- Authentication -> JWT
- Profile management -> See if there is JWT
  - See there profile
  - Change name from there profile

*/

use axum::{
    Router,
    routing::{get, post},
};
use tokio::net::TcpListener;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().ok();
    
    // Set up database connection pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    // Create router with shared state
    let app = Router::new()
        // Profile endpoints
        .route("/v1/auth", post(authenticate_user))
        .route("/v1/reg", post(register_user))
        .route("/v1/updprf", post(update_profile))
        .route("/v1/getprofile", get(get_profile))
        .with_state(pool);

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    println!("Listening on port: 8080");
    axum::serve(listener, app).await.unwrap();
}

async fn authenticate_user() {}
async fn register_user() {}
async fn update_profile() {}
async fn get_profile() {}
