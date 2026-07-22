pub mod admin;
pub mod auth;
pub mod classes;
pub mod finance;
pub mod inventory;
pub mod library;
pub mod todos;
pub mod transport;

use axum::Router;
use crate::AppState;

pub fn api_router(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/auth",      auth::router())
        .nest("/admin",     admin::admin_router())
        .nest("/students",  admin::students_router())
        .nest("/classes",   classes::router())
        .nest("/inventory", inventory::router())
        .nest("/library",   library::router(state.clone()))
        .nest("/transport", transport::router())
        .nest("/finance",   finance::router())
        .nest("/todos",     todos::router())
}
