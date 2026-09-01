pub mod handlers;
pub mod models;

use axum::{
    routing::{get, patch},
    Router,
};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::get_classes).post(handlers::create_class))
        .route("/:id", patch(handlers::edit_class).put(handlers::edit_class).delete(handlers::remove_class))
}
