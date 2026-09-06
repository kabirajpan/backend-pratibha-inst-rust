// backend-rust/src/modules/whatsapp/mod.rs
pub mod models;
pub mod dto;
pub mod service;
pub mod handlers;
pub mod routes;

pub use routes::router;
