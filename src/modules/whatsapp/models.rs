use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SendWhatsAppPayload {
    pub phone_number: String,
    pub message: String,
    pub template_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppStatusResponse {
    pub enabled: bool,
    pub provider: String,
    pub phone_number_id_configured: bool,
    pub api_key_configured: bool,
}
