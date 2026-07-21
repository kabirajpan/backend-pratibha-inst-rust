use crate::errors::AppError;

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let cost = if cfg!(debug_assertions) { 4 } else { 12 };
    bcrypt::hash(password, cost).map_err(AppError::from)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    bcrypt::verify(password, hash).map_err(AppError::from)
}
