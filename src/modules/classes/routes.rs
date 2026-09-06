// backend-rust/src/modules/classes/routes.rs
use axum::{
    routing::{get, patch},
    Router,
};
use crate::AppState;
use super::handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::get_classes).post(handlers::create_class))
        .route("/:id", patch(handlers::edit_class).put(handlers::edit_class).delete(handlers::remove_class))
}
