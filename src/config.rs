use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub jwt_access_secret: String,
    pub jwt_refresh_secret: String,
    pub jwt_access_expiry: String,
    pub jwt_refresh_expiry: String,
    pub client_origin: String,
    pub node_env: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    pub resend_api_key: Option<String>,
    pub sms_provider: String,
    pub sms_api_key: String,
    pub sms_route: String,
    pub sms_enabled: bool,
    pub whatsapp_enabled: bool,
    pub whatsapp_provider: String,
    pub whatsapp_api_key: String,
    pub whatsapp_phone_number_id: String,
    pub whatsapp_account_sid: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let port = env::var("PORT")
            .unwrap_or_else(|_| "5000".to_string())
            .parse::<u16>()
            .expect("PORT must be a valid u16 number");

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in .env");

        let jwt_access_secret = env::var("JWT_ACCESS_SECRET")
            .expect("JWT_ACCESS_SECRET must be set in .env");

        let jwt_refresh_secret = env::var("JWT_REFRESH_SECRET")
            .expect("JWT_REFRESH_SECRET must be set in .env");

        let jwt_access_expiry = env::var("JWT_ACCESS_EXPIRY")
            .unwrap_or_else(|_| "15m".to_string());

        let jwt_refresh_expiry = env::var("JWT_REFRESH_EXPIRY")
            .unwrap_or_else(|_| "7d".to_string());

        let client_origin = env::var("CLIENT_ORIGIN")
            .unwrap_or_else(|_| "https://frontend-pratibha-inst.vercel.app".to_string());

        let node_env = env::var("NODE_ENV")
            .unwrap_or_else(|_| "development".to_string());

        let smtp_host = env::var("SMTP_HOST")
            .unwrap_or_else(|_| "smtp.gmail.com".to_string());

        let smtp_port = env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse::<u16>()
            .unwrap_or(587);

        let smtp_username = env::var("SMTP_USERNAME")
            .unwrap_or_default();

        let smtp_password = env::var("SMTP_PASSWORD")
            .unwrap_or_default();

        let smtp_from_email = env::var("SMTP_FROM_EMAIL")
            .unwrap_or_default();

        let smtp_from_name = env::var("SMTP_FROM_NAME")
            .unwrap_or_else(|_| "Pratibha Institute ERP".to_string());

        let resend_api_key = env::var("RESEND_API_KEY").ok().filter(|s| !s.trim().is_empty());

        let sms_provider = env::var("SMS_PROVIDER")
            .unwrap_or_else(|_| "fast2sms".to_string());

        let sms_api_key = env::var("SMS_API_KEY")
            .unwrap_or_default();

        let sms_route = env::var("SMS_ROUTE")
            .unwrap_or_else(|_| "q".to_string());

        let sms_enabled = env::var("SMS_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);

        let whatsapp_enabled = env::var("WHATSAPP_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);

        let whatsapp_provider = env::var("WHATSAPP_PROVIDER")
            .unwrap_or_else(|_| "meta".to_string());

        let whatsapp_api_key = env::var("WHATSAPP_API_KEY")
            .unwrap_or_default();

        let whatsapp_phone_number_id = env::var("WHATSAPP_PHONE_NUMBER_ID")
            .unwrap_or_default();

        let whatsapp_account_sid = env::var("WHATSAPP_ACCOUNT_SID")
            .ok()
            .filter(|s| !s.trim().is_empty());

        Config {
            port,
            database_url,
            jwt_access_secret,
            jwt_refresh_secret,
            jwt_access_expiry,
            jwt_refresh_expiry,
            client_origin,
            node_env,
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            smtp_from_email,
            smtp_from_name,
            resend_api_key,
            sms_provider,
            sms_api_key,
            sms_route,
            sms_enabled,
            whatsapp_enabled,
            whatsapp_provider,
            whatsapp_api_key,
            whatsapp_phone_number_id,
            whatsapp_account_sid,
        }
    }

    pub fn is_prod(&self) -> bool {
        self.node_env == "production"
    }
}
