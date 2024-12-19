use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FlamingoPrice {
    pub symbol: String,
    #[serde(rename = "unwrappedSymbol")]
    pub unwrapped_symbol: String,
    pub hash: String,
    pub usd_price: f64,
    pub block_index: Option<u64>,
    pub timestamp: Option<i64>,
}
