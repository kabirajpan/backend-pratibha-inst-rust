// backend-rust/src/modules/whatsapp/routes.rs
use axum::{
    routing::{get, post},
    Router,
};
use crate::AppState;
use super::handlers::{get_status_handler, send_whatsapp_handler};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/status", get(get_status_handler))
        .route("/send", post(send_whatsapp_handler))
        .with_state(state)
}
