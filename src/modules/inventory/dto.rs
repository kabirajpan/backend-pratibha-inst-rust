// src/modules/inventory/dto.rs
use serde::Deserialize;
use uuid::Uuid;
use chrono::NaiveDate;

#[derive(Debug, Deserialize)]
pub struct CreateCategoryPayload {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryPayload {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateItemPayload {
    pub category_id: Uuid,
    pub name: String,
    pub required_qty: Option<i32>,
    pub available_qty: Option<i32>,
    pub unit_price: Option<f64>,
    pub low_stock_threshold: Option<i32>,
    pub unit: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateItemPayload {
    pub category_id: Option<Uuid>,
    pub name: Option<String>,
    pub required_qty: Option<i32>,
    pub available_qty: Option<i32>,
    pub unit_price: Option<f64>,
    pub low_stock_threshold: Option<i32>,
    pub unit: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetItemsQuery {
    pub search: Option<String>,
    pub category_id: Option<Uuid>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct IssueItemPayload {
    pub item_id: Uuid,
    pub qty: i32,
    pub issued_to: String,
    pub issue_date: Option<NaiveDate>,
    pub remarks: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReturnItemPayload {
    pub id: Uuid,
    pub return_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct ImportItemsPayload {
    pub items: Vec<ImportItemRow>,
}

#[derive(Debug, Deserialize)]
pub struct ImportItemRow {
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub name: String,
    pub required_qty: Option<i32>,
    pub available_qty: Option<i32>,
    pub unit_price: Option<f64>,
    pub unit: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImportIssuesPayload {
    pub issues: Vec<ImportIssueRow>,
}

#[derive(Debug, Deserialize)]
pub struct ImportIssueRow {
    pub item_name: String,
    pub qty: Option<i32>,
    pub issued_to: String,
    pub issue_date: Option<NaiveDate>,
    pub return_date: Option<NaiveDate>,
    pub status: Option<String>,
    pub remarks: Option<String>,
}
