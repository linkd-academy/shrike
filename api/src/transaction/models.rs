use crate::block::models::Witness;
use crate::shared::models::{Address, Hash160};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct TxDataList {
    pub address: String,
    pub as_sender: Vec<TxData>,
    pub as_participant: Vec<TxData>,
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
pub struct Transfer {
    pub contract: Hash160,
    pub from: Address,
    pub to: Address,
    pub amount: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub index: u64,
    pub hash: String,
    pub block_index: u64,
    pub timestamp: u64,
    pub vm_state: String,
    pub size: u32,
    pub version: u8,
    pub nonce: u64,
    pub sender: String,
    pub sysfee: String,
    pub netfee: String,
    pub valid_until: u64,
    pub signers: String,
    pub script: String,
    pub witnesses: Vec<Witness>,
    pub stack_result: String,
    pub notifications: Vec<Notification>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Notification {
    pub id: Option<u64>,
    pub contract: String,
    pub eventname: String,
    pub state: State,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    #[serde(rename = "type")]
    pub _type: String,
    pub value: Vec<StateValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StateValue {
    #[serde(rename = "type")]
    pub _type: String,
    pub value: Option<serde_json::Value>,
}
