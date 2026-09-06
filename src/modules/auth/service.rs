// backend-rust/src/modules/auth/service.rs
//! Auth Business Logic & Domain Service Layer

use sqlx::PgPool;
use uuid::Uuid;
use crate::config::Config;
use crate::errors::AppError;
use crate::utils::jwt::{sign_jwt, verify_jwt};
use crate::utils::password::{hash_password, verify_password};
use super::models::{
    ChangePasswordPayload, LoginPayload, LoginResponseData,
    RefreshResponseData, RegisterPayload, UserResponse, UserRole,
};
use super::repository;

/// Register a new user with validation, password hashing, and onboarding email
pub async fn register_user(
    pool: &PgPool,
    config: &Config,
    payload: RegisterPayload,
) -> Result<UserResponse, AppError> {
    payload.validate()?;

    // Check if email already registered
    let existing = repository::find_by_email(pool, &payload.email)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if existing.is_some() {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    let role = payload.role.clone().unwrap_or(UserRole::Student);

    // If role is Student, verify pre-registration in students directory
    if role == UserRole::Student {
        let student_profile = repository::find_student_by_email(pool, &payload.email)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if student_profile.is_none() {
            return Err(AppError::Forbidden(
                "Your email is not pre-registered in the student directory. Please contact the administrator.".to_string(),
            ));
        }
    }

    let password_hash = hash_password(&payload.password)?;

    let user = repository::create_user(
        pool,
        &payload.name,
        &payload.email,
        &password_hash,
        &role,
        payload.sub_role.as_ref(),
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to register user: {}", e)))?;

    // Send onboarding email for staff
    if role != UserRole::Student {
        let role_title = payload.sub_role.as_ref().map(|sr| format!("{:?}", sr)).unwrap_or_else(|| format!("{:?}", role));
        let portal_url = format!("{}/login", config.client_origin.trim_end_matches('/'));
        let html = crate::modules::email::service::build_staff_welcome_html(
            &payload.name,
            &role_title,
            &payload.email,
            &payload.password,
            &portal_url,
        );
        crate::modules::email::service::send_email_async(
            config.clone(),
            payload.email.clone(),
            format!("Staff Onboarding - Pratibha ERP ({})", role_title),
            html,
        );
    }

    Ok(UserResponse::from(user))
}

/// Authenticate user and issue JWT tokens
pub async fn authenticate(
    pool: &PgPool,
    config: &Config,
    payload: LoginPayload,
) -> Result<(LoginResponseData, String), AppError> {
    payload.validate()?;

    let user = repository::find_by_email(pool, &payload.email)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let user = match user {
        Some(u) if u.is_active => u,
        _ => return Err(AppError::Unauthorized("Invalid email or password".to_string())),
    };

    let is_match = verify_password(&payload.password, &user.password_hash)?;
    if !is_match {
        return Err(AppError::Unauthorized("Invalid email or password".to_string()));
    }

    let access_token = sign_jwt(
        user.id,
        user.role.clone(),
        user.sub_role.clone(),
        &config.jwt_access_secret,
        &config.jwt_access_expiry,
    )?;

    let refresh_token = sign_jwt(
        user.id,
        user.role.clone(),
        user.sub_role.clone(),
        &config.jwt_refresh_secret,
        &config.jwt_refresh_expiry,
    )?;

    let user_resp = UserResponse::from(user);
    let login_data = LoginResponseData {
        access_token,
        user: user_resp,
    };

    Ok((login_data, refresh_token))
}

/// Refresh access token from a valid refresh token
pub async fn refresh_access_token(
    config: &Config,
    refresh_token: &str,
) -> Result<RefreshResponseData, AppError> {
    let claims = verify_jwt(refresh_token, &config.jwt_refresh_secret)
        .map_err(|_| AppError::Unauthorized("Invalid or expired refresh token".to_string()))?;

    let access_token = sign_jwt(
        claims.id,
        claims.role,
        claims.sub_role,
        &config.jwt_access_secret,
        &config.jwt_access_expiry,
    )?;

    Ok(RefreshResponseData { access_token })
}

/// Retrieve authenticated user profile
pub async fn get_current_user(pool: &PgPool, user_id: Uuid) -> Result<UserResponse, AppError> {
    let user = repository::find_by_id(pool, user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(UserResponse::from(user))
}

/// Retrieve staff members list
pub async fn get_staff_members(pool: &PgPool) -> Result<Vec<UserResponse>, AppError> {
    let staff = repository::find_all_staff(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch staff: {}", e)))?;

    Ok(staff.into_iter().map(UserResponse::from).collect())
}

/// Update user password with current password verification
pub async fn change_password(
    pool: &PgPool,
    user_id: Uuid,
    payload: ChangePasswordPayload,
) -> Result<(), AppError> {
    payload.validate()?;

    let user = repository::find_by_id(pool, user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    if let Some(ref current_password) = payload.current_password {
        let is_match = verify_password(current_password, &user.password_hash)?;
        if !is_match {
            return Err(AppError::BadRequest("Current password does not match".to_string()));
        }
    }

    let hashed_new_password = hash_password(&payload.new_password)?;

    repository::update_password_hash(pool, user_id, &hashed_new_password)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update password: {}", e)))
}
