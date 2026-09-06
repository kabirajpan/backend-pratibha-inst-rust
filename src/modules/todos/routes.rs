// backend-rust/src/modules/todos/routes.rs
use axum::{
    routing::{get, patch, post},
    Router,
};
use crate::AppState;
use super::handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:module", get(handlers::get_todos).post(handlers::create_todo))
        .route("/:module/clear-completed", post(handlers::clear_completed_todos))
        .route("/:module/:id", patch(handlers::edit_todo).delete(handlers::remove_todo))
}
