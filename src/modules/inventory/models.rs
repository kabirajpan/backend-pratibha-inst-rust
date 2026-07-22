// src/modules/inventory/models.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, NaiveDate, Utc};

// Inventory Categories
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct InventoryCategoryRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub total_items: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CategorySimpleRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

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

// Inventory Items
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct InventoryItemRow {
    pub id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub required_qty: i32,
    pub available_qty: i32,
    pub unit_price: f64,
    pub low_stock_threshold: i32,
    pub unit: String,
    pub description: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub category_name: String,
    pub entry_by_name: Option<String>,
    pub is_low_stock: bool,
    pub total_value: f64,
    pub total_count: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SingleItemRow {
    pub id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub required_qty: i32,
    pub available_qty: i32,
    pub unit_price: f64,
    pub low_stock_threshold: i32,
    pub unit: String,
    pub description: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub category_name: String,
    pub entry_by_name: Option<String>,
    pub is_low_stock: bool,
    pub total_value: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RawItemRow {
    pub id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub required_qty: i32,
    pub available_qty: i32,
    pub unit_price: f64,
    pub low_stock_threshold: i32,
    pub unit: String,
    pub description: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct LowStockItemRow {
    pub id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub required_qty: i32,
    pub available_qty: i32,
    pub unit_price: f64,
    pub low_stock_threshold: i32,
    pub unit: String,
    pub description: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub category_name: String,
    pub shortage: i32,
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

// Inventory Issues
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct InventoryIssueRow {
    pub id: Uuid,
    pub item_id: Uuid,
    pub qty: i32,
    pub issued_to: String,
    pub issue_date: NaiveDate,
    pub return_date: Option<NaiveDate>,
    pub status: String,
    pub entry_by: Option<Uuid>,
    pub remarks: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub item_name: String,
    pub item_unit: String,
    pub entry_by_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RawIssueRow {
    pub id: Uuid,
    pub item_id: Uuid,
    pub qty: i32,
    pub issued_to: String,
    pub issue_date: NaiveDate,
    pub return_date: Option<NaiveDate>,
    pub status: String,
    pub entry_by: Option<Uuid>,
    pub remarks: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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

#[derive(Debug, Serialize, FromRow)]
pub struct StatsRow {
    pub total_items: i64,
    pub total_categories: i64,
    pub low_stock_items: i64,
    pub total_value: f64,
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
