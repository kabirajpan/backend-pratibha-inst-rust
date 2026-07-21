pub mod handlers;
pub mod models;
pub mod transport;
pub mod hostel;
pub mod tuition;

use axum::{
    routing::get,
    Router,
};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        // Fees collections
        .route(
            "/fees/:id_or_type",
            get(handlers::get_fee_records)
                .post(handlers::create_fee_record)
                .patch(handlers::edit_fee_record)
                .delete(handlers::remove_fee_record),
        )
        .route("/fee-record/:id", get(handlers::get_fee_record))
        // Expenses ledger
        .route("/expenses",       get(handlers::get_expenses).post(handlers::create_expense))
        .route("/expenses/:id",   get(handlers::get_expense).patch(handlers::edit_expense).delete(handlers::remove_expense))
}
