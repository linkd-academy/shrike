use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct DailyAddressBalance {
    pub block_index: u64,
    pub date: String,
    pub timestamp: u64,
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
