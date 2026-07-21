use crate::errors::AppError;
use crate::modules::auth::models::{UserRole, UserSubRole};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub id: Uuid,
    pub role: UserRole,
    #[serde(default)]
    pub sub_role: Option<UserSubRole>,
    pub exp: i64,
}

pub fn parse_expiry(expiry_str: &str) -> i64 {
    let expiry_str = expiry_str.trim();
    if expiry_str.is_empty() {
        return 900; // 15 minutes default
    }
    let unit = expiry_str.chars().last().unwrap_or('s');
    let val_str = &expiry_str[0..expiry_str.len() - 1];
    let val = val_str.parse::<i64>().unwrap_or(0);
    match unit {
        's' => val,
        'm' => val * 60,
        'h' => val * 3600,
        'd' => val * 86_400,
        _ => val,
    }
}

pub fn sign_jwt(
    id: Uuid,
    role: UserRole,
    sub_role: Option<UserSubRole>,
    secret: &str,
    expiry_str: &str,
) -> Result<String, AppError> {
    let seconds = parse_expiry(expiry_str);
    let expiration = Utc::now()
        .checked_add_signed(Duration::seconds(seconds))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        id,
        role,
        sub_role,
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, AppError> {
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims)
}
