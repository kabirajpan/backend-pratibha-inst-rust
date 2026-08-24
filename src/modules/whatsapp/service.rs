use tracing::{error, info};
use crate::config::Config;

/// Asynchronously send a WhatsApp message without blocking HTTP handler execution
pub fn send_whatsapp_async(config: Config, to_phone: String, message: String, template_name: Option<String>) {
    tokio::spawn(async move {
        if !config.whatsapp_enabled {
            info!("Skipping WhatsApp send: WhatsApp notifications are disabled in configuration");
            return;
        }

        let clean_phone: String = to_phone
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();

        if clean_phone.is_empty() {
            info!("Skipping WhatsApp send: empty phone number provided");
            return;
        }

        // Format phone number with country code (defaults to 91 for 10-digit Indian numbers)
        let formatted_phone = if clean_phone.len() == 10 {
            format!("91{}", clean_phone)
        } else {
            clean_phone
        };

        if config.whatsapp_api_key.trim().is_empty() && config.node_env == "production" {
            error!("Skipping WhatsApp send: WHATSAPP_API_KEY is not configured");
            return;
        }

        let client = reqwest::Client::new();
        let provider = config.whatsapp_provider.to_lowercase();

        match provider.as_str() {
            "meta" | "facebook" => {
                let phone_number_id = if config.whatsapp_phone_number_id.trim().is_empty() {
                    "YOUR_PHONE_NUMBER_ID"
                } else {
                    config.whatsapp_phone_number_id.trim()
                };

                let url = format!(
                    "https://graph.facebook.com/v18.0/{}/messages",
                    phone_number_id
                );

                let payload = if let Some(ref t_name) = template_name {
                    serde_json::json!({
                        "messaging_product": "whatsapp",
                        "to": formatted_phone,
                        "type": "template",
                        "template": {
                            "name": t_name,
                            "language": {
                                "code": "en_US"
                            }
                        }
                    })
                } else {
                    serde_json::json!({
                        "messaging_product": "whatsapp",
                        "recipient_type": "individual",
                        "to": formatted_phone,
                        "type": "text",
                        "text": {
                            "preview_url": false,
                            "body": message
                        }
                    })
                };

                match client
                    .post(&url)
                    .header("Authorization", format!("Bearer {}", config.whatsapp_api_key.trim()))
                    .header("Content-Type", "application/json")
                    .json(&payload)
                    .send()
                    .await
                {
                    Ok(res) if res.status().is_success() => {
                        info!("✅ WhatsApp message successfully sent to '+{}' via Meta Cloud API", formatted_phone);
                    }
                    Ok(res) => {
                        let status = res.status();
                        let err_body = res.text().await.unwrap_or_default();
                        error!("❌ Meta WhatsApp API error {}: {}", status, err_body);
                    }
                    Err(e) => {
                        error!("❌ Meta WhatsApp network connection failed for '+{}': {:?}", formatted_phone, e);
                    }
                }
            }
            "ultramsg" => {
                let url = format!(
                    "https://api.ultramsg.com/{}/messages/chat",
                    config.whatsapp_phone_number_id.trim()
                );

                let payload = serde_json::json!({
                    "token": config.whatsapp_api_key.trim(),
                    "to": formatted_phone,
                    "body": message
                });

                match client
                    .post(&url)
                    .header("Content-Type", "application/json")
                    .json(&payload)
                    .send()
                    .await
                {
                    Ok(res) if res.status().is_success() => {
                        info!("✅ WhatsApp message sent to '+{}' via UltraMsg", formatted_phone);
                    }
                    Ok(res) => {
                        let status = res.status();
                        let err_body = res.text().await.unwrap_or_default();
                        error!("❌ UltraMsg API error {}: {}", status, err_body);
                    }
                    Err(e) => {
                        error!("❌ UltraMsg connection failed for '+{}': {:?}", formatted_phone, e);
                    }
                }
            }
            _ => {
                info!(
                    "📱 [WhatsApp Gateway Log] Provider: '{}' | To: '+{}' | Message: {}",
                    config.whatsapp_provider, formatted_phone, message
                );
            }
        }
    });
}

// ─── WHATSAPP TEMPLATE BUILDERS ──────────────────────────────────────────────

/// Build structured WhatsApp message for fee payment receipt confirmation
pub fn build_fee_receipt_whatsapp(
    student_name: &str,
    receipt_no: &str,
    fee_type: &str,
    amount_paid: f64,
    remaining_due: f64
) -> String {
    let fee_title = match fee_type.to_lowercase().as_str() {
        "hostel" => "Hostel",
        "transport" => "Transport",
        "library" => "Library",
        _ => "Tuition"
    };

    format!(
        "🧾 *Fee Payment Receipt Confirmation*\n\n\
        Dear *{}*,\n\n\
        We have received your payment of *₹{:.2}* for *{} Fee*.\n\n\
        • *Receipt No:* {}\n\
        • *Amount Paid:* ₹{:.2}\n\
        • *Remaining Balance:* ₹{:.2}\n\n\
        Thank you,\n*Pratibha Institute of Nursing*",
        student_name, amount_paid, fee_title, receipt_no, amount_paid, remaining_due
    )
}

/// Build WhatsApp message for student enrollment & credentials
pub fn build_student_welcome_whatsapp(
    student_name: &str,
    student_id: &str,
    default_password: &str,
    portal_url: &str
) -> String {
    let url_clean = if portal_url.trim().is_empty() {
        "https://frontend-pratibha-inst.vercel.app/login"
    } else {
        portal_url
    };

    format!(
        "🎓 *Welcome to Pratibha Institute of Nursing*\n\n\
        Dear *{}*,\n\n\
        Your student portal account has been created!\n\n\
        • *Student ID:* {}\n\
        • *Default Password:* {}\n\
        • *Portal Link:* {}\n\n\
        Please log in and update your password immediately.",
        student_name, student_id, default_password, url_clean
    )
}

/// Build WhatsApp message for staff onboarding credentials
pub fn build_staff_welcome_whatsapp(
    staff_name: &str,
    role_title: &str,
    login_email: &str,
    default_password: &str,
    portal_url: &str
) -> String {
    let url_clean = if portal_url.trim().is_empty() {
        "https://frontend-pratibha-inst.vercel.app/login"
    } else {
        portal_url
    };

    format!(
        "🏥 *Welcome to Pratibha ERP Portal*\n\n\
        Hello *{}*,\n\n\
        Your *{}* account is now active.\n\n\
        • *Login Email:* {}\n\
        • *Password:* {}\n\
        • *Portal Link:* {}\n\n\
        Thank you for joining our team!",
        staff_name, role_title, login_email, default_password, url_clean
    )
}

/// Helper to query student phone and trigger fee payment receipt WhatsApp message
pub async fn trigger_fee_receipt_whatsapp(
    config: &Config,
    db: &sqlx::PgPool,
    student_id: &str,
    receipt_no: &str,
    fee_type: &str,
    amount_paid: f64,
    remaining_due: f64
) {
    if amount_paid <= 0.0 || !config.whatsapp_enabled {
        return;
    }

    let clean_id = student_id.trim();
    let query_str = "SELECT name, phone, parent_phone FROM students WHERE LOWER(TRIM(student_id)) = LOWER(TRIM($1))";
    let row = sqlx::query_as::<_, StudentPhoneInfo>(query_str)
        .bind(clean_id)
        .fetch_optional(db)
        .await;

    if let Ok(Some(s)) = row {
        let target_phone = s.phone.or(s.parent_phone);
        if let Some(phone) = target_phone {
            if !phone.trim().is_empty() {
                let msg = build_fee_receipt_whatsapp(
                    &s.name,
                    receipt_no,
                    fee_type,
                    amount_paid,
                    remaining_due
                );
                send_whatsapp_async(config.clone(), phone, msg, None);
            }
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct StudentPhoneInfo {
    name: String,
    phone: Option<String>,
    parent_phone: Option<String>,
}
