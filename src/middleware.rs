use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use uuid::Uuid;
use crate::errors::AppError;
use crate::modules::auth::models::{UserRole, UserSubRole};
use crate::AppState;
use crate::utils::jwt::verify_jwt;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub role: UserRole,
    pub sub_role: Option<UserSubRole>,
}

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok());

        let auth_header = auth_header.ok_or_else(|| {
            AppError::Unauthorized("Not authenticated".to_string())
        })?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized("Not authenticated".to_string()));
        }

        let token = &auth_header[7..];

        let claims = verify_jwt(token, &state.config.jwt_access_secret)
            .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;

        Ok(AuthUser {
            id: claims.id,
            role: claims.role,
            sub_role: claims.sub_role,
        })
    }
}

impl AuthUser {
    pub fn authorize(&self, roles: &[UserRole]) -> Result<(), AppError> {
        if roles.contains(&self.role) {
            Ok(())
        } else {
            Err(AppError::Forbidden("Forbidden".to_string()))
        }
    }

    pub fn authorize_sub_role(&self, sub_roles: &[UserSubRole]) -> Result<(), AppError> {
        if self.role == UserRole::Admin {
            return Ok(());
        }
        if let Some(ref sub_role) = self.sub_role {
            if sub_roles.contains(sub_role) {
                return Ok(());
            }
        }
        Err(AppError::Forbidden("Forbidden".to_string()))
    }
}
