// backend-rust/src/modules/auth/routes.rs
use axum::{
    routing::{get, post},
    Router,
};
use crate::AppState;
use super::handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/refresh", post(handlers::refresh))
        .route("/logout", post(handlers::logout))
        .route("/me", get(handlers::me))
        .route("/staff", get(handlers::get_staff))
        .route("/change-password", post(handlers::change_password))
}
