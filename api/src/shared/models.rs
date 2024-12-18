use serde::{Deserialize, Serialize};
use serde_json::Value;

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
pub struct Transaction {
    pub index: u64,
    pub hash: String,
    pub block_index: u64,
    pub vm_state: String,
    pub size: u32,
    pub version: u8,
    pub nonce: u64,
    pub sender: String,
    pub sysfee: String,
    pub netfee: String,
    pub valid_until: u64,
    pub signers: Value,
    pub script: String,
    pub witnesses: Value,
    pub stack_result: Value,
    pub notifications: Vec<Notification>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Notification {
    pub id: Option<u64>,
    pub contract: String,
    pub eventname: String,
    pub state: State,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct State {
    #[serde(rename = "type")]
    pub _type: String,
    pub value: Vec<StateValue>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StateValue {
    #[serde(rename = "type")]
    pub _type: String,
    pub value: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransactionList {
    pub transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Transfer {
    pub contract: Hash160,
    pub from: Address,
    pub to: Address,
    pub amount: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TxData {
    pub txid: String,
    pub time: u64, // unix timestamp, extra call to set it until I modify the db to store block time for transactions
    pub sysfee: f64,
    pub netfee: f64,
    pub nep17_transfers: Vec<Transfer>,
    pub nep11_transfers: Vec<Transfer>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TxDataList {
    pub address: String,
    pub as_sender: Vec<TxData>,
    pub as_participant: Vec<TxData>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Event {
    pub contract: Hash160,
    pub eventname: String,
    pub state: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DailyAddressBalance {
    pub block_index: u64,
    pub date: String,
    pub address: String,
    pub token_contract: String,
    pub balance: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DailyTokenPrice {
    pub block_index: u64,
    pub date: String,
    pub token_contract: String,
    pub price: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DailyContractUsage {
    pub date: String,
    pub contract: String,
    pub usage: u32,
}
