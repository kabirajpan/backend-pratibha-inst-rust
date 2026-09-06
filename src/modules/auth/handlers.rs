// backend-rust/src/modules/auth/handlers.rs
//! Auth HTTP Presentation Layer (Axum Handlers)

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthUser;
use super::models::{
    ApiMessageResponse, ApiResponse, ChangePasswordPayload, LoginPayload,
    RegisterPayload, UserRole,
};
use super::service;

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<impl IntoResponse, AppError> {
    let user_resp = service::register_user(&state.db, &state.config, payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: user_resp,
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<LoginPayload>,
) -> Result<impl IntoResponse, AppError> {
    let (login_data, refresh_token) = service::authenticate(&state.db, &state.config, payload).await?;

    let cookie = Cookie::build(("refreshToken", refresh_token))
        .path("/api/auth")
        .http_only(true)
        .secure(state.config.is_prod())
        .same_site(if state.config.is_prod() { SameSite::None } else { SameSite::Lax })
        .max_age(time::Duration::days(7))
        .build();

    let jar = jar.add(cookie);

    Ok((
        jar,
        Json(ApiResponse {
            success: true,
            data: login_data,
        }),
    ))
}

pub async fn refresh(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    let token = jar
        .get("refreshToken")
        .map(|c| c.value())
        .ok_or_else(|| AppError::Unauthorized("No refresh token provided".to_string()))?;

    let refresh_data = service::refresh_access_token(&state.config, token).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: refresh_data,
    }))
}

pub async fn logout(
    _auth_user: AuthUser,
    jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    let cookie = Cookie::build(("refreshToken", ""))
        .path("/api/auth")
        .max_age(time::Duration::seconds(0))
        .build();

    let jar = jar.remove(cookie);

    Ok((
        jar,
        Json(ApiMessageResponse {
            success: true,
            message: "Logged out".to_string(),
        }),
    ))
}

pub async fn me(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let user_resp = service::get_current_user(&state.db, auth_user.id).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: user_resp,
    }))
}

pub async fn get_staff(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    auth_user.authorize(&[UserRole::Admin])?;

    let staff_resp = service::get_staff_members(&state.db).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: staff_resp,
    }))
}

pub async fn change_password(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<ChangePasswordPayload>,
) -> Result<impl IntoResponse, AppError> {
    service::change_password(&state.db, auth_user.id, payload).await?;

    Ok(Json(ApiMessageResponse {
        success: true,
        message: "Password updated successfully".to_string(),
    }))
}
