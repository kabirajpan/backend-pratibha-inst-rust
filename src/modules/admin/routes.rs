// backend-rust/src/modules/admin/routes.rs
use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use crate::AppState;
use super::handlers;

pub fn admin_router() -> Router<AppState> {
    Router::new()
        // User account management (admin only)
        .route("/users",            get(handlers::get_all_users))
        .route("/users/:id/toggle", patch(handlers::toggle_user_active))
        .route("/users/:id",        delete(handlers::delete_user))
        .route("/audit-logs",       get(handlers::get_audit_logs).post(handlers::create_audit_log))
        .route("/settings",         get(handlers::get_system_settings).patch(handlers::update_system_settings))
}

pub fn students_router() -> Router<AppState> {
    Router::new()
        // Students (read: any authenticated; write: admin only)
        .route("/",                 get(handlers::get_students).post(handlers::create_student))
        .route("/import",           post(handlers::import_students))
        .route("/credentials/send", post(handlers::send_student_credentials))
        .route("/credentials/reset", post(handlers::reset_student_passwords))
        .route("/:id",              get(handlers::get_student).patch(handlers::edit_student).delete(handlers::remove_student))
}
