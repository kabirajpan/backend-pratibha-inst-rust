// backend-rust/src/modules/finance/mod.rs
pub mod models;
pub mod dto;
pub mod repository;
pub mod service;
pub mod transport;
pub mod hostel;
pub mod tuition;
pub mod handlers;
pub mod routes;

pub use routes::router;
