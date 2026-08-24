pub mod models;
pub mod service;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use crate::AppState;
use crate::errors::AppError;
use self::models::{SendWhatsAppPayload, WhatsAppStatusResponse};
use self::service::send_whatsapp_async;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/status", get(get_status_handler))
        .route("/send", post(send_whatsapp_handler))
        .with_state(state)
}

async fn get_status_handler(
    State(state): State<AppState>,
) -> Result<Json<WhatsAppStatusResponse>, AppError> {
    let cfg = &state.config;
    Ok(Json(WhatsAppStatusResponse {
        enabled: cfg.whatsapp_enabled,
        provider: cfg.whatsapp_provider.clone(),
        phone_number_id_configured: !cfg.whatsapp_phone_number_id.trim().is_empty(),
        api_key_configured: !cfg.whatsapp_api_key.trim().is_empty(),
    }))
}

async fn send_whatsapp_handler(
    State(state): State<AppState>,
    Json(payload): Json<SendWhatsAppPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    if payload.phone_number.trim().is_empty() {
        return Err(AppError::BadRequest("Phone number is required".into()));
    }
    if payload.message.trim().is_empty() {
        return Err(AppError::BadRequest("Message body is required".into()));
    }

    send_whatsapp_async(
        state.config.clone(),
        payload.phone_number.clone(),
        payload.message.clone(),
        payload.template_name,
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("WhatsApp message dispatch queued for {}", payload.phone_number)
    })))
}
