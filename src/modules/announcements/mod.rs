// backend-rust/src/modules/announcements/mod.rs
pub mod models;
pub mod dto;
pub mod repository;
pub mod service;
pub mod handlers;
pub mod routes;

pub use routes::{announcements_router, notifications_router};
