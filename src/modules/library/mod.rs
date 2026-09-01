pub mod handlers;
pub mod models;

use axum::{
    body::Body,
    extract::{Path, State},
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use uuid::Uuid;
use crate::AppState;
use crate::errors::AppError;
use crate::modules::auth::models::UserSubRole;

pub async fn library_guard(
    auth_user: crate::middleware::AuthUser,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    auth_user.authorize_sub_role(&[UserSubRole::LibraryManager])?;
    Ok(next.run(request).await)
}

pub fn router(state: AppState) -> Router<AppState> {
    let classes_routes = crate::modules::classes::router();
    let courses_routes = crate::modules::courses::router();

    let library_core_routes = Router::new()
        // Stats & Activity
        .route("/stats", get(handlers::get_stats))
        .route("/activity", get(handlers::get_activity))
        // Books
        .route("/books", get(handlers::get_books).post(handlers::create_book))
        .route("/books/import", post(handlers::import_books))
        .route("/books/:id", get(handlers::get_book).patch(handlers::edit_book).delete(handlers::remove_book))
        // Members
        .route("/members", get(handlers::get_members).post(handlers::create_member))
        .route("/members/import", post(handlers::import_members))
        .route("/members/:id", get(handlers::get_member).patch(handlers::edit_member))
        // Issues
        .route("/issues", get(handlers::get_issues))
        .route("/issue", post(handlers::issue_book))
        .route("/return", post(handlers::return_book))
        .route("/return/import", post(handlers::import_returns))
        .route("/issues/:id/fine", patch(handlers::update_fine))
        // Settings
        .route("/settings", get(handlers::get_settings).patch(handlers::edit_settings))
        // Todos nested wrappers
        .route("/todos", get(todo_get_handler).post(todo_post_handler))
        .route("/todos/clear-completed", post(todo_clear_handler))
        .route("/todos/:id", patch(todo_patch_handler).delete(todo_delete_handler))
        .route_layer(axum::middleware::from_fn_with_state(state, library_guard));

    Router::new()
        .nest("/classes", classes_routes)
        .nest("/courses", courses_routes)
        .nest("/", library_core_routes)
}

// ─── TODOS NESTED WRAPPERS ───────────────────────────────

async fn todo_get_handler(
    state: State<AppState>,
    auth_user: crate::middleware::AuthUser,
) -> impl IntoResponse {
    crate::modules::todos::handlers::get_todos(state, auth_user, Path("library".to_string())).await
}

async fn todo_post_handler(
    state: State<AppState>,
    auth_user: crate::middleware::AuthUser,
    Json(payload): Json<crate::modules::todos::models::CreateTodoPayload>,
) -> impl IntoResponse {
    crate::modules::todos::handlers::create_todo(state, auth_user, Path("library".to_string()), Json(payload)).await
}

async fn todo_patch_handler(
    state: State<AppState>,
    auth_user: crate::middleware::AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<crate::modules::todos::models::UpdateTodoPayload>,
) -> impl IntoResponse {
    crate::modules::todos::handlers::edit_todo(state, auth_user, Path(("library".to_string(), id)), Json(payload)).await
}

async fn todo_delete_handler(
    state: State<AppState>,
    auth_user: crate::middleware::AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    crate::modules::todos::handlers::remove_todo(state, auth_user, Path(("library".to_string(), id))).await
}

async fn todo_clear_handler(
    state: State<AppState>,
    auth_user: crate::middleware::AuthUser,
) -> impl IntoResponse {
    crate::modules::todos::handlers::clear_completed_todos(state, auth_user, Path("library".to_string())).await
}
