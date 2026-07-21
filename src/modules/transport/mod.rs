pub mod handlers;
pub mod models;

use axum::{
    routing::{get, post},
    Router,
};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        // Vehicles
        .route("/vehicles",       get(handlers::get_vehicles).post(handlers::create_vehicle))
        .route("/vehicles/:id",   get(handlers::get_vehicle).patch(handlers::edit_vehicle).delete(handlers::remove_vehicle))
        // Expenses
        .route("/expenses",       get(handlers::get_expenses).post(handlers::create_expense))
        .route("/expenses/:id",   get(handlers::get_expense).patch(handlers::edit_expense).delete(handlers::remove_expense))
        // Students
        .route("/students",         get(handlers::get_transport_students).post(handlers::create_transport_student))
        .route("/students/import",  post(handlers::import_transport_students))
        .route("/students/:id",     get(handlers::get_transport_student).patch(handlers::edit_transport_student).delete(handlers::remove_transport_student))
}
