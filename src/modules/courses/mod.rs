pub mod handlers;
pub mod models;

use axum::{routing::get, Router};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::get_courses).post(handlers::create_course))
        .route("/:id", axum::routing::put(handlers::edit_course).delete(handlers::remove_course))
}
