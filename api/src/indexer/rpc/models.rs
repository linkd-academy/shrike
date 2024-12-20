use serde::{Deserialize, Serialize, Serializer};
use thiserror::Error;

use crate::transaction::models::{Notification, StateValue};

#[derive(Deserialize, Debug, Clone)]
pub enum NeoParam {
    String(String),
    Integer(u64),
    Boolean(bool),
    Array(Vec<NeoParam>),
    Object(Vec<(String, NeoParam)>),
}

impl Serialize for NeoParam {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            NeoParam::String(value) => serializer.serialize_str(value),
            NeoParam::Integer(value) => serializer.serialize_u64(*value),
            NeoParam::Boolean(value) => serializer.serialize_bool(*value),
            NeoParam::Array(value) => value.serialize(serializer),
            NeoParam::Object(map) => {
                use serde::ser::SerializeMap;

                let mut map_serializer = serializer.serialize_map(Some(map.len()))?;
                for (key, val) in map {
                    map_serializer.serialize_entry(key, val)?;
                }
                map_serializer.end()
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Vec<NeoParam>,
    pub id: u32,
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Request error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Deserialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Deserialize, Debug)]
pub struct RpcResponse<T> {
    pub jsonrpc: String,
    pub id: u32,
    pub result: T,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BlockResult {
    pub hash: String,
    pub size: u32,
    pub version: u8,
    pub merkleroot: String,
    pub time: u64,
    pub nonce: String,
    pub index: u64,
    pub primary: u8,
    pub nextconsensus: String,
    pub witnesses: Vec<Witness>,
    pub tx: Vec<TransactionResult>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TransactionResult {
    pub hash: String,
    pub blockhash: Option<String>,
    pub size: u32,
    #[serde(default)]
    pub timestamp: u64,
    pub version: u8,
    pub nonce: u64,
    pub sender: String,
    pub sysfee: String,
    pub netfee: String,
    pub validuntilblock: u64,
    pub signers: Vec<Signer>,
    pub script: String,
    pub witnesses: Vec<Witness>,
}

#[derive(Deserialize, Debug, Clone)]
pub enum AppLogResult {
    BlockAppLogResult(BlockAppLogResult),
    TransactionAppLogResult(TransactionAppLogResult),
}

#[derive(Deserialize, Debug, Clone)]
pub struct BlockAppLogResult {
    pub blockhash: String,
    pub executions: Vec<Execution>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TransactionAppLogResult {
    pub txid: String,
    pub executions: Vec<Execution>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Execution {
    #[serde(default)]
    pub trigger: String,
    #[serde(default)]
    pub vmstate: String,
    #[serde(default)]
    pub exception: Option<String>,
    pub gasconsumed: String,
    pub stack: Vec<StateValue>,
    pub notifications: Vec<Notification>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Witness {
    pub invocation: String,
    pub verification: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Signer {
    pub account: String,
    pub scopes: String,
    pub allowedcontracts: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Contract {
    pub block_index: u64,
    pub hash: String,
    pub contract_type: String,
}
