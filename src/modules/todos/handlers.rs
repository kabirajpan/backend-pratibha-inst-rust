use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::modules::auth::models::{ApiResponse, ApiMessageResponse};
use super::models::{Todo, CreateTodoPayload, UpdateTodoPayload};

pub async fn get_todos(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Path(module): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let todos = sqlx::query_as::<_, Todo>(
        "SELECT * FROM todos WHERE module = $1 ORDER BY created_at DESC"
    )
    .bind(&module)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: todos,
    }))
}

pub async fn create_todo(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(module): Path<String>,
    Json(payload): Json<CreateTodoPayload>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    let parsed_due_date = match &payload.due_date {
        Some(d) if !d.is_empty() => Some(
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("Invalid due date format".to_string()))?
        ),
        _ => None,
    };

    let priority = payload.priority.unwrap_or_else(|| "medium".to_string());

    let todo = sqlx::query_as::<_, Todo>(
        r#"
        INSERT INTO todos (user_id, module, text, priority, due_date, category)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#
    )
    .bind(Some(auth_user.id))
    .bind(&module)
    .bind(&payload.text)
    .bind(&priority)
    .bind(parsed_due_date)
    .bind(&payload.category)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: todo,
    }))
}

pub async fn edit_todo(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Path((module, id)): Path<(String, Uuid)>,
    Json(payload): Json<UpdateTodoPayload>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    let existing = sqlx::query_as::<_, Todo>(
        "SELECT * FROM todos WHERE id = $1 AND module = $2"
    )
    .bind(id)
    .bind(&module)
    .fetch_optional(&state.db)
    .await?;

    let mut existing = match existing {
        Some(t) => t,
        None => return Err(AppError::NotFound("Todo not found".to_string())),
    };

    if let Some(text) = payload.text {
        existing.text = text;
    }
    if let Some(completed) = payload.completed {
        existing.completed = completed;
    }
    if let Some(priority) = payload.priority {
        existing.priority = priority;
    }
    if let Some(due_date_str) = payload.due_date {
        existing.due_date = if due_date_str.is_empty() {
            None
        } else {
            Some(
                chrono::NaiveDate::parse_from_str(&due_date_str, "%Y-%m-%d")
                    .map_err(|_| AppError::BadRequest("Invalid due date format".to_string()))?
            )
        };
    }
    if let Some(category) = payload.category {
        existing.category = if category.is_empty() { None } else { Some(category) };
    }

    let updated = sqlx::query_as::<_, Todo>(
        r#"
        UPDATE todos
        SET text = $1, completed = $2, priority = $3, due_date = $4, category = $5, updated_at = now()
        WHERE id = $6 AND module = $7
        RETURNING *
        "#
    )
    .bind(&existing.text)
    .bind(existing.completed)
    .bind(&existing.priority)
    .bind(existing.due_date)
    .bind(&existing.category)
    .bind(id)
    .bind(&module)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ApiResponse {
        success: true,
        data: updated,
    }))
}

pub async fn remove_todo(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Path((module, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let deleted = sqlx::query(
        "DELETE FROM todos WHERE id = $1 AND module = $2 RETURNING id"
    )
    .bind(id)
    .bind(&module)
    .fetch_optional(&state.db)
    .await?;

    if deleted.is_none() {
        return Err(AppError::NotFound("Todo not found".to_string()));
    }

    Ok(Json(ApiMessageResponse {
        success: true,
        message: "Todo deleted".to_string(),
    }))
}

pub async fn clear_completed_todos(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Path(module): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query("DELETE FROM todos WHERE module = $1 AND completed = true")
        .bind(&module)
        .execute(&state.db)
        .await?;

    Ok(Json(ApiMessageResponse {
        success: true,
        message: "Completed todos cleared".to_string(),
    }))
}
