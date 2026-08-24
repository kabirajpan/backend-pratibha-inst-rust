pub mod admin;
pub mod announcements;
pub mod auth;
pub mod classes;
pub mod email;
pub mod finance;
pub mod hostel;
pub mod inventory;
pub mod library;
pub mod sms;
pub mod todos;
pub mod transport;
pub mod whatsapp;

use axum::Router;
use crate::AppState;

pub fn api_router(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/auth",          auth::router())
        .nest("/admin",         admin::admin_router())
        .nest("/students",      admin::students_router())
        .nest("/classes",       classes::router())
        .nest("/inventory",     inventory::router())
        .nest("/library",       library::router(state.clone()))
        .nest("/transport",     transport::router())
        .nest("/hostel",        hostel::router())
        .nest("/finance",       finance::router())
        .nest("/todos",         todos::router())
        .nest("/announcements", announcements::announcements_router())
        .nest("/notifications", announcements::notifications_router())
        .nest("/whatsapp",      whatsapp::router(state.clone()))
}

