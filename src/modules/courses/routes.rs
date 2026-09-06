// backend-rust/src/modules/courses/routes.rs
use axum::{
    routing::{get, patch},
    Router,
};
use crate::AppState;
use super::handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::get_courses).post(handlers::create_course))
        .route("/:id", patch(handlers::edit_course).put(handlers::edit_course).delete(handlers::remove_course))
}
