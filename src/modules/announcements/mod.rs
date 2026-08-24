pub mod handlers;
pub mod models;

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use crate::AppState;

pub fn announcements_router() -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::create_announcement).get(handlers::get_announcements))
        .route("/permissions", get(handlers::get_broadcast_permissions).put(handlers::update_broadcast_permissions))
        .route("/:id", delete(handlers::delete_announcement))
}

pub fn notifications_router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::get_user_notifications))
        .route("/read-all", put(handlers::mark_all_notifications_read))
        .route("/:id/read", put(handlers::mark_notification_read))
}
