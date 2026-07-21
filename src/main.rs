pub mod config;
pub mod db;
pub mod errors;
pub mod middleware;
pub mod modules;
pub mod utils;

use axum::{
    http::{HeaderValue, Method, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::db::create_pool;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: db::DbPool,
}

#[tokio::main]
async fn main() {
    // Initialize tracing/logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend_rust=info,sqlx=warn,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configurations
    let config = Config::from_env();
    let port = config.port;

    // Establish DB connection pool
    tracing::info!("Connecting to database...");
    let db = match create_pool(&config).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Database connection failed: {}", e);
            std::process::exit(1);
        }
    };
    tracing::info!("Database connection pool established successfully.");

    let state = AppState {
        config: config.clone(),
        db,
    };

    // Configure CORS
    let origin = config.client_origin.parse::<HeaderValue>().unwrap_or_else(|_| {
        HeaderValue::from_static("http://localhost:3000")
    });

    let cors = CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
        ])
        .allow_credentials(true);

    // Build routes using modular api_router
    let api_routes = modules::api_router(state.clone());

    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/api", api_routes)
        .fallback(fallback_handler)
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
        )
        .layer(cors)
        .with_state(state);

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Failed to bind server listener to {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    tracing::info!("✅ Server running on http://{}", addr);
    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("Server error: {}", e);
    }
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "message": "Pratibha Backend Running 🚀"
    }))
}

async fn fallback_handler() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "success": false,
            "message": "Route not found"
        })),
    )
}
