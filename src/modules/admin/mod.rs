pub mod handlers;
pub mod models;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use crate::AppState;

pub fn admin_router() -> Router<AppState> {
    Router::new()
        // User account management (admin only)
        .route("/users",            get(handlers::get_all_users))
        .route("/users/:id/toggle", patch(handlers::toggle_user_active))
        .route("/users/:id",        delete(handlers::delete_user))
}

pub fn students_router() -> Router<AppState> {
    Router::new()
        // Students (read: any authenticated; write: admin only)
        .route("/",        get(handlers::get_students).post(handlers::create_student))
        .route("/import",  post(handlers::import_students))
        .route("/:id",     get(handlers::get_student).patch(handlers::edit_student).delete(handlers::remove_student))
}
