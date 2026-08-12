pub mod handlers;
pub mod models;

use axum::{
    routing::{get, post},
    Router,
};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        // Rooms
        .route("/rooms", get(handlers::get_hostel_rooms).post(handlers::create_hostel_room))
        .route("/rooms/:id", axum::routing::patch(handlers::edit_hostel_room).delete(handlers::remove_hostel_room))
        // Students / Residents
        .route("/students", get(handlers::get_hostel_students).post(handlers::create_hostel_student))
        .route("/students/:id", axum::routing::patch(handlers::edit_hostel_student).delete(handlers::remove_hostel_student))
}
