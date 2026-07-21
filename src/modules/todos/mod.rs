pub mod handlers;
pub mod models;

use axum::{
    routing::{get, patch, post},
    Router,
};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:module", get(handlers::get_todos).post(handlers::create_todo))
        .route("/:module/clear-completed", post(handlers::clear_completed_todos))
        .route("/:module/:id", patch(handlers::edit_todo).delete(handlers::remove_todo))
}
