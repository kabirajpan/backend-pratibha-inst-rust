use tracing::{error, info};
use crate::config::Config;

/// Asynchronously send an SMS via Fast2SMS / Twilio / MSG91 without blocking HTTP handler execution
pub fn send_sms_async(config: Config, to_phone: String, message: String) {
    tokio::spawn(async move {
        if !config.sms_enabled {
            info!("Skipping SMS send: SMS notifications are disabled in configuration");
            return;
        }

        let clean_phone: String = to_phone
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();

        // Extract last 10 digits for Indian phone numbers if country code (+91) is attached
        let phone_number = if clean_phone.len() > 10 {
            clean_phone[clean_phone.len() - 10..].to_string()
        } else {
            clean_phone
        };

        if phone_number.len() != 10 {
            info!("Skipping SMS send: invalid 10-digit mobile number '{}'", to_phone);
            return;
        }

        if config.sms_api_key.trim().is_empty() {
            error!("Skipping SMS send: SMS_API_KEY is not configured");
            return;
        }

        let client = reqwest::Client::new();

        if config.sms_provider.to_lowercase() == "fast2sms" {
            let payload = serde_json::json!({
                "route": config.sms_route,
                "message": message,
                "numbers": phone_number,
            });

            match client
                .post("https://www.fast2sms.com/dev/bulkV2")
                .header("authorization", config.sms_api_key.trim())
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
            {
                Ok(res) if res.status().is_success() => {
                    info!("✅ SMS successfully sent to '{}' via Fast2SMS", phone_number);
                }
                Ok(res) => {
                    let status = res.status();
                    let err_body = res.text().await.unwrap_or_default();
                    error!("❌ Fast2SMS API error {}: {}", status, err_body);
                }
                Err(e) => {
                    error!("❌ Fast2SMS network connection failed for {}: {:?}", phone_number, e);
                }
            }
        } else {
            info!("Unsupported SMS provider '{}'. Fast2SMS is active by default.", config.sms_provider);
        }
    });
}

// ─── SMS TEMPLATE BUILDERS ───────────────────────────────────────────────────

/// Build concise SMS for student enrollment & login credentials
pub fn build_student_welcome_sms(
    student_name: &str,
    student_id: &str,
    default_password: &str,
    portal_url: &str
) -> String {
    let url_clean = if portal_url.trim().is_empty() { "https://frontend-pratibha-inst.vercel.app/login" } else { portal_url };
    format!(
        "Dear {}, Welcome to Pratibha Inst of Nursing! ID: {}, Pass: {}. Portal: {}",
        student_name, student_id, default_password, url_clean
    )
}

/// Build concise SMS for fee payment receipt confirmation
pub fn build_fee_receipt_sms(
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
        "Dear {}, Rs.{:.0} received for {} Fee (Receipt #{})! Due: Rs.{:.0}. Pratibha Inst.",
        student_name, amount_paid, fee_title, receipt_no, remaining_due
    )
}

/// Build concise SMS for staff onboarding credentials
pub fn build_staff_welcome_sms(
    staff_name: &str,
    role_title: &str,
    login_email: &str,
    default_password: &str,
    portal_url: &str
) -> String {
    let url_clean = if portal_url.trim().is_empty() { "https://frontend-pratibha-inst.vercel.app/login" } else { portal_url };
    format!(
        "Hello {}, Your {} account at Pratibha ERP is active! User: {}, Pass: {}. Portal: {}",
        staff_name, role_title, login_email, default_password, url_clean
    )
}

/// Helper to query student phone and trigger fee payment receipt SMS
pub async fn trigger_fee_receipt_sms(
    config: &Config,
    db: &sqlx::PgPool,
    student_id: &str,
    receipt_no: &str,
    fee_type: &str,
    amount_paid: f64,
    remaining_due: f64
) {
    if amount_paid <= 0.0 || !config.sms_enabled {
        return; // Skip SMS for zero-amount initial allocation records
    }

    let clean_id = student_id.trim();
    let query_str = "SELECT name, phone, parent_phone FROM students WHERE LOWER(TRIM(student_id)) = LOWER(TRIM($1))";
    let row = sqlx::query_as::<_, StudentPhoneInfo>(query_str)
        .bind(clean_id)
        .fetch_optional(db)
        .await;

    if let Ok(Some(s)) = row {
        // Prefer student phone, fallback to parent phone
        let target_phone = s.phone.or(s.parent_phone);
        if let Some(phone) = target_phone {
            if !phone.trim().is_empty() {
                let msg = build_fee_receipt_sms(
                    &s.name,
                    receipt_no,
                    fee_type,
                    amount_paid,
                    remaining_due
                );
                send_sms_async(config.clone(), phone, msg);
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
