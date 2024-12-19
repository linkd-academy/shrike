use anyhow::Result;
use reqwest::Client as ReqwestClient;

use super::models::FlamingoPrice;

pub struct FlamingoClient {
    client: ReqwestClient,
    base_url: String,
}

impl FlamingoClient {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            client: ReqwestClient::new(),
            base_url: base_url.unwrap_or("https://neo-api.b-cdn.net").to_string(),
        }
    }

    pub async fn get_prices_from_block(&self, block_number: u64) -> Result<Vec<FlamingoPrice>> {
        let url = format!(
            "{}/flamingo/live-data/prices/from-block/{}",
            self.base_url, block_number
        );

        let response = self
            .client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Erro ao buscar dados: {}",
                response.status()
            ));
        }

        let prices = response.json::<Vec<FlamingoPrice>>().await?;
        Ok(prices)
    }

    pub async fn get_latest_prices(&self) -> Result<Vec<FlamingoPrice>> {
        let url = format!("{}/flamingo/live-data/prices/latest", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Erro ao buscar dados: {}",
                response.status()
            ));
        }

        let prices = response.json::<Vec<FlamingoPrice>>().await?;
        Ok(prices)
    }
}
