use lettre::message::header::ContentType;
use lettre::message::{Mailbox, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use tracing::{error, info};
use crate::config::Config;

/// Asynchronously send an HTML email without blocking HTTP request execution.
pub fn send_email_async(config: Config, to: String, subject: String, body_html: String) {
    tokio::spawn(async move {
        let recipient_email = to.trim().to_string();
        if recipient_email.is_empty() || !recipient_email.contains('@') {
            info!("Skipping email send: invalid recipient address '{}'", recipient_email);
            return;
        }

        let from_email = match config.smtp_from_email.parse() {
            Ok(e) => e,
            Err(err) => {
                error!("Failed to parse SMTP_FROM_EMAIL '{}': {:?}", config.smtp_from_email, err);
                return;
            }
        };

        let to_email = match recipient_email.parse() {
            Ok(e) => e,
            Err(err) => {
                error!("Failed to parse recipient email address '{}': {:?}", recipient_email, err);
                return;
            }
        };

        let from_mailbox = Mailbox::new(Some(config.smtp_from_name.clone()), from_email);
        let to_mailbox = Mailbox::new(None, to_email);

        let email = match Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(&subject)
            .header(ContentType::TEXT_HTML)
            .body(body_html)
        {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to build email message for {}: {:?}", recipient_email, e);
                return;
            }
        };

        let creds = Credentials::new(config.smtp_username.clone(), config.smtp_password.clone());

        let mailer_builder = if config.smtp_port == 465 {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host)
        };

        let mailer = match mailer_builder {
            Ok(builder) => builder.port(config.smtp_port).credentials(creds).build(),
            Err(e) => {
                error!("Failed to build SMTP transport for {}:{}: {:?}", config.smtp_host, config.smtp_port, e);
                return;
            }
        };

        match mailer.send(email).await {
            Ok(_) => info!("✅ Email successfully sent to '{}' [Subject: '{}']", recipient_email, subject),
            Err(e) => error!("❌ Failed to send email to '{}': {:?}", recipient_email, e),
        }
    });
}

/// Helper to trigger fee receipt email if student has an email registered
pub async fn trigger_fee_receipt_email(
    config: &Config,
    db: &sqlx::PgPool,
    student_id: &str,
    receipt_no: &str,
    fee_type: &str,
    amount_paid: f64,
    remaining_due: f64,
    payment_mode: &str,
    payment_date: &str
) {
    if amount_paid <= 0.0 {
        return; // Don't send receipt for 0 amount initial due balance records
    }

    let clean_id = student_id.trim();
    let query_str = "SELECT name, email FROM students WHERE LOWER(TRIM(student_id)) = LOWER(TRIM($1))";
    let row = sqlx::query_as::<_, StudentEmailInfo>(query_str)
        .bind(clean_id)
        .fetch_optional(db)
        .await;

    if let Ok(Some(s)) = row {
        if let Some(to_email) = s.email {
            let email_str = to_email.trim().to_string();
            if email_str.contains('@') {
                let html = build_fee_receipt_html(
                    &s.name,
                    clean_id,
                    receipt_no,
                    fee_type,
                    amount_paid,
                    remaining_due,
                    payment_mode,
                    payment_date
                );
                send_email_async(
                    config.clone(),
                    email_str,
                    format!("Fee Payment Receipt #{} - Pratibha Institute", receipt_no),
                    html
                );
            }
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct StudentEmailInfo {
    name: String,
    email: Option<String>,
}

// ─── HTML TEMPLATE BUILDERS ───────────────────────────────────────────────────

/// Build responsive HTML welcome email for registered students
pub fn build_student_welcome_html(
    student_name: &str,
    student_id: &str,
    login_email: &str,
    default_password: &str,
    class_name: &str,
    portal_url: &str
) -> String {
    let login_href = if portal_url.trim().is_empty() { "http://localhost:3000/login" } else { portal_url };
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Welcome to Pratibha Institute</title>
</head>
<body style="font-family: Arial, sans-serif; background-color: #f8fafc; margin: 0; padding: 20px; color: #1e293b;">
    <div style="max-width: 600px; margin: 0 auto; background: #ffffff; border-radius: 12px; overflow: hidden; border: 1px solid #e2e8f0; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.05);">
        
        <!-- Header -->
        <div style="background-color: #0f172a; padding: 28px; text-align: center;">
            <h1 style="color: #ffffff; margin: 0; font-size: 20px; letter-spacing: 0.5px; text-transform: uppercase;">Pratibha Institute of Nursing</h1>
            <p style="color: #38bdf8; margin: 6px 0 0 0; font-size: 13px; font-weight: bold;">Student ERP Portal Access</p>
        </div>

        <!-- Body -->
        <div style="padding: 32px 28px;">
            <h2 style="font-size: 16px; color: #0f172a; margin-top: 0;">Dear {student_name},</h2>
            <p style="font-size: 14px; line-height: 1.6; color: #475569;">
                Welcome to <strong>Pratibha Institute of Nursing</strong>! Your official student account has been created successfully for class <strong>{class_name}</strong>.
            </p>

            <!-- Account Details Card -->
            <div style="background-color: #f1f5f9; border-left: 4px solid #2563eb; padding: 18px; border-radius: 6px; margin: 24px 0;">
                <p style="margin: 0 0 8px 0; font-size: 12px; font-weight: bold; color: #64748b; text-transform: uppercase; letter-spacing: 0.5px;">Your Portal Login Credentials</p>
                <p style="margin: 4px 0; font-size: 14px;"><strong>Student ID:</strong> <span style="font-family: monospace; color: #2563eb;">{student_id}</span></p>
                <p style="margin: 4px 0; font-size: 14px;"><strong>Login User:</strong> <span style="font-family: monospace; color: #0f172a;">{login_email}</span></p>
                <p style="margin: 4px 0; font-size: 14px;"><strong>Default Password:</strong> <span style="font-family: monospace; color: #dc2626;">{default_password}</span></p>
            </div>

            <p style="font-size: 13px; color: #64748b; line-height: 1.5;">
                You can log into your student dashboard to track attendance, fee schedules, hostel, transport, exam results, and library books.
            </p>

            <div style="text-align: center; margin: 32px 0 16px 0;">
                <a href="{login_href}" style="background-color: #2563eb; color: #ffffff; text-decoration: none; padding: 12px 28px; border-radius: 6px; font-weight: bold; font-size: 14px; display: inline-block;">
                    Access Student Portal &rarr;
                </a>
            </div>
        </div>

        <!-- Footer -->
        <div style="background-color: #f8fafc; padding: 16px 28px; text-align: center; border-t: 1px solid #e2e8f0; font-size: 12px; color: #94a3b8;">
            &copy; 2026 Pratibha Institute of Nursing, Khuteri New Raipur C.G. All rights reserved.
        </div>
    </div>
</body>
</html>"#
    )
}

/// Build responsive HTML receipt email for fee payments (Tuition, Hostel, Transport)
pub fn build_fee_receipt_html(
    student_name: &str,
    student_id: &str,
    receipt_no: &str,
    fee_type: &str,
    amount_paid: f64,
    remaining_due: f64,
    payment_mode: &str,
    payment_date: &str
) -> String {
    let fee_type_title = match fee_type.to_lowercase().as_str() {
        "hostel" => "Hostel & Accommodation Fee",
        "transport" => "Transport & Bus Service Fee",
        "library" => "Library Fee & Fine",
        _ => "Tuition & Academic Fee"
    };

    let formatted_paid = format!("₹{:.2}", amount_paid);
    let formatted_due = format!("₹{:.2}", remaining_due);

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Fee Payment Confirmation</title>
</head>
<body style="font-family: Arial, sans-serif; background-color: #f8fafc; margin: 0; padding: 20px; color: #1e293b;">
    <div style="max-width: 600px; margin: 0 auto; background: #ffffff; border-radius: 12px; overflow: hidden; border: 1px solid #e2e8f0; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.05);">
        
        <!-- Header -->
        <div style="background-color: #0f172a; padding: 28px; text-align: center;">
            <h1 style="color: #ffffff; margin: 0; font-size: 18px; letter-spacing: 0.5px; text-transform: uppercase;">Pratibha Institute of Nursing</h1>
            <p style="color: #10b981; margin: 6px 0 0 0; font-size: 13px; font-weight: bold;">Official Fee Payment Receipt</p>
        </div>

        <!-- Body -->
        <div style="padding: 32px 28px;">
            <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 2px solid #f1f5f9; pb: 16px; margin-bottom: 20px;">
                <div>
                    <h2 style="font-size: 15px; color: #0f172a; margin: 0;">Receipt No: <span style="color: #2563eb; font-family: monospace;">{receipt_no}</span></h2>
                    <p style="margin: 4px 0 0 0; font-size: 12px; color: #64748b;">Date: {payment_date}</p>
                </div>
            </div>

            <p style="font-size: 14px; color: #334155; margin-bottom: 20px;">
                Dear <strong>{student_name}</strong> (Student ID: <span style="font-family: monospace;">{student_id}</span>),
                <br>We have received your payment for <strong>{fee_type_title}</strong>. Details of the transaction are outlined below:
            </p>

            <!-- Receipt Breakdown Table -->
            <table style="width: 100%; border-collapse: collapse; margin: 20px 0; font-size: 13px;">
                <thead>
                    <tr style="background-color: #f8fafc; border-bottom: 1px solid #e2e8f0; text-align: left; color: #64748b;">
                        <th style="padding: 10px; font-weight: bold;">Description</th>
                        <th style="padding: 10px; text-align: right; font-weight: bold;">Details / Amount</th>
                    </tr>
                </thead>
                <tbody>
                    <tr style="border-bottom: 1px solid #f1f5f9;">
                        <td style="padding: 12px 10px; color: #475569;">Fee Category</td>
                        <td style="padding: 12px 10px; text-align: right; font-weight: bold; color: #0f172a;">{fee_type_title}</td>
                    </tr>
                    <tr style="border-bottom: 1px solid #f1f5f9;">
                        <td style="padding: 12px 10px; color: #475569;">Payment Method</td>
                        <td style="padding: 12px 10px; text-align: right; font-weight: bold; color: #2563eb;">{payment_mode}</td>
                    </tr>
                    <tr style="border-bottom: 1px solid #f1f5f9; background-color: #f0fdf4;">
                        <td style="padding: 12px 10px; font-weight: bold; color: #166534;">Amount Received</td>
                        <td style="padding: 12px 10px; text-align: right; font-weight: bold; color: #166534; font-size: 16px; font-family: monospace;">{formatted_paid}</td>
                    </tr>
                    <tr style="border-bottom: 1px solid #f1f5f9;">
                        <td style="padding: 12px 10px; color: #475569;">Outstanding Balance Remaining</td>
                        <td style="padding: 12px 10px; text-align: right; font-weight: bold; color: #dc2626; font-family: monospace;">{formatted_due}</td>
                    </tr>
                </tbody>
            </table>

            <p style="font-size: 12px; color: #64748b; margin-top: 24px;">
                * This is a computer-generated digital payment receipt. You can log into your student portal anytime to view and download full transaction statements.
            </p>
        </div>

        <!-- Footer -->
        <div style="background-color: #f8fafc; padding: 16px 28px; text-align: center; border-t: 1px solid #e2e8f0; font-size: 12px; color: #94a3b8;">
            Accounts & Finance Department • Pratibha Institute of Nursing
        </div>
    </div>
</body>
</html>"#
    )
}

/// Build responsive HTML welcome email for new staff accounts
pub fn build_staff_welcome_html(
    staff_name: &str,
    role_title: &str,
    login_email: &str,
    default_password: &str,
    portal_url: &str
) -> String {
    let login_href = if portal_url.trim().is_empty() { "http://localhost:3000/login" } else { portal_url };
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Staff Account Invitation</title>
</head>
<body style="font-family: Arial, sans-serif; background-color: #f8fafc; margin: 0; padding: 20px; color: #1e293b;">
    <div style="max-width: 600px; margin: 0 auto; background: #ffffff; border-radius: 12px; overflow: hidden; border: 1px solid #e2e8f0; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.05);">
        
        <!-- Header -->
        <div style="background-color: #0f172a; padding: 28px; text-align: center;">
            <h1 style="color: #ffffff; margin: 0; font-size: 18px; letter-spacing: 0.5px; text-transform: uppercase;">Pratibha Institute ERP</h1>
            <p style="color: #38bdf8; margin: 6px 0 0 0; font-size: 13px; font-weight: bold;">Staff Portal Onboarding</p>
        </div>

        <!-- Body -->
        <div style="padding: 32px 28px;">
            <h2 style="font-size: 16px; color: #0f172a; margin-top: 0;">Hello {staff_name},</h2>
            <p style="font-size: 14px; line-height: 1.6; color: #475569;">
                Your staff account has been created on the <strong>Pratibha Institute Administrative ERP</strong> with the role of <strong>{role_title}</strong>.
            </p>

            <div style="background-color: #f1f5f9; border-left: 4px solid #0284c7; padding: 18px; border-radius: 6px; margin: 24px 0;">
                <p style="margin: 0 0 8px 0; font-size: 12px; font-weight: bold; color: #64748b; text-transform: uppercase;">Staff Credentials</p>
                <p style="margin: 4px 0; font-size: 14px;"><strong>Email:</strong> <span style="font-family: monospace; color: #0f172a;">{login_email}</span></p>
                <p style="margin: 4px 0; font-size: 14px;"><strong>Password:</strong> <span style="font-family: monospace; color: #dc2626;">{default_password}</span></p>
            </div>

            <div style="text-align: center; margin: 32px 0 16px 0;">
                <a href="{login_href}" style="background-color: #0f172a; color: #ffffff; text-decoration: none; padding: 12px 28px; border-radius: 6px; font-weight: bold; font-size: 14px; display: inline-block;">
                    Log In to Staff Portal &rarr;
                </a>
            </div>
        </div>
    </div>
</body>
</html>"#
    )
}
