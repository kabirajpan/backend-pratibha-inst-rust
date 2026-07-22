// src/modules/inventory/mod.rs
pub mod handlers;
pub mod models;

use axum::{
    routing::{get, post, patch},
    Router,
};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        // Stats & Low stock
        .route("/stats",             get(handlers::get_stats))
        .route("/low-stock",          get(handlers::get_low_stock))
        
        // Categories
        .route("/categories",        get(handlers::get_categories).post(handlers::create_category))
        .route("/categories/:id",    patch(handlers::edit_category).delete(handlers::remove_category))
        
        // Items
        .route("/items",             get(handlers::get_items).post(handlers::create_item))
        .route("/items/import",      post(handlers::import_items))
        .route("/items/:id",         get(handlers::get_item).patch(handlers::edit_item).delete(handlers::remove_item))
        
        // Issues & Returns
        .route("/issues",            get(handlers::get_issues))
        .route("/issue",             post(handlers::issue_item))
        .route("/return",            post(handlers::return_item))
        .route("/issues/import",     post(handlers::import_issues))
}
