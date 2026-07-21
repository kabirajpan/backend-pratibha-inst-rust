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
use crate::utils::jwt::{sign_jwt, verify_jwt};
use crate::utils::password::{hash_password, verify_password};
use super::models::{
    ApiMessageResponse, ApiResponse, ChangePasswordPayload, LoginPayload,
    LoginResponseData, RefreshResponseData, RegisterPayload, User, UserResponse, UserRole,
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    // Check if email already registered in users
    let email_exists = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&state.db)
        .await?;

    if email_exists.is_some() {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    let role = payload.role.clone().unwrap_or(UserRole::Student);

    if role == UserRole::Student {
        let student_profile = sqlx::query("SELECT student_id FROM students WHERE email = $1")
            .bind(&payload.email)
            .fetch_optional(&state.db)
            .await?;

        if student_profile.is_none() {
            return Err(AppError::Forbidden(
                "Your email is not pre-registered in the student directory. Please contact the administrator.".to_string(),
            ));
        }
    }

    let password_hash = hash_password(&payload.password)?;

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (name, email, password_hash, role, sub_role)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, email, password_hash, role, sub_role, is_active, created_at, updated_at
        "#
    )
    .bind(&payload.name)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&role)
    .bind(&payload.sub_role)
    .fetch_one(&state.db)
    .await?;

    let response_data = UserResponse::from(user);

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: response_data,
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<LoginPayload>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&state.db)
        .await?;

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
        &state.config.jwt_access_secret,
        &state.config.jwt_access_expiry,
    )?;

    let refresh_token = sign_jwt(
        user.id,
        user.role.clone(),
        user.sub_role.clone(),
        &state.config.jwt_refresh_secret,
        &state.config.jwt_refresh_expiry,
    )?;

    let cookie = Cookie::build(("refreshToken", refresh_token))
        .path("/api/auth")
        .http_only(true)
        .secure(state.config.is_prod())
        .same_site(if state.config.is_prod() { SameSite::None } else { SameSite::Lax })
        .max_age(time::Duration::days(7))
        .build();

    let jar = jar.add(cookie);

    let user_resp = UserResponse::from(user);
    let login_data = LoginResponseData {
        access_token,
        user: user_resp,
    };

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

    let claims = verify_jwt(token, &state.config.jwt_refresh_secret)
        .map_err(|_| AppError::Unauthorized("Invalid or expired refresh token".to_string()))?;

    let access_token = sign_jwt(
        claims.id,
        claims.role,
        claims.sub_role,
        &state.config.jwt_access_secret,
        &state.config.jwt_access_expiry,
    )?;

    let refresh_data = RefreshResponseData { access_token };

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
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, email, password_hash, role, sub_role, is_active, created_at, updated_at FROM users WHERE id = $1"
    )
    .bind(auth_user.id)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    let user_resp = UserResponse::from(user);

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

    let staff = sqlx::query_as::<_, User>(
        "SELECT id, name, email, password_hash, role, sub_role, is_active, created_at, updated_at FROM users WHERE role = 'staff' ORDER BY name ASC"
    )
    .fetch_all(&state.db)
    .await?;

    let staff_resp: Vec<UserResponse> = staff.into_iter().map(UserResponse::from).collect();

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
    payload.validate()?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(auth_user.id)
        .fetch_optional(&state.db)
        .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    if let Some(ref current_password) = payload.current_password {
        let is_match = verify_password(current_password, &user.password_hash)?;
        if !is_match {
            return Err(AppError::BadRequest("Current password does not match".to_string()));
        }
    }

    let hashed_new_password = hash_password(&payload.new_password)?;

    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(hashed_new_password)
        .bind(auth_user.id)
        .execute(&state.db)
        .await?;

    Ok(Json(ApiMessageResponse {
        success: true,
        message: "Password updated successfully".to_string(),
    }))
}
