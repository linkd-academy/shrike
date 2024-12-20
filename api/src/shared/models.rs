use serde::{Deserialize, Serialize};

pub const GAS_PRECISION: f64 = 100000000.0;
pub const FUSDT_PRECISION: f64 = 1000000.0;

pub type Hash160 = String;
pub type Address = String;

pub const PAGE_DEFAULT: u32 = 0;
pub const PER_PAGE_DEFAULT: u32 = 100;
pub const PER_PAGE_LIMIT: u32 = 1000;

#[derive(Deserialize)]
pub struct PaginationAndFilterParams {
    pub page: Option<u32>,       // Page
    pub per_page: Option<u32>,   // Number of items per page
    pub order: Option<String>,   // "asc" or "desc"
    pub sort_by: Option<String>, // Column to order

    pub date_init: Option<String>, // Filter date init
    pub date_end: Option<String>,  // Filter date end
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Event {
    pub contract: Hash160,
    pub eventname: String,
    pub state: serde_json::Value,
}
