use anyhow::Result;
use reqwest::Client as ReqwestClient;

use crate::config::AppConfig;

use super::method::{
    GetApplicationLog, GetBlock, GetBlockCount, InvokeFunction, InvokeFunctionHistoric, RpcMethod,
};
use super::models::{
    BlockAppLogResult, BlockResult, ClientError, Execution, NeoParam, RpcRequest, RpcResponse,
    TransactionAppLogResult, TransactionResult,
};

pub struct Client {
    client: ReqwestClient,
    base_url: String,
}

impl Client {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            client: ReqwestClient::new(),
            base_url: config.node_path.clone(),
        }
    }

    pub async fn send_request<T: RpcMethod, R: serde::de::DeserializeOwned>(
        &self,
        method: T,
    ) -> Result<R, ClientError> {
        self.send_request_with_log(method, false).await
    }

    pub async fn send_request_with_log<T: RpcMethod, R: serde::de::DeserializeOwned>(
        &self,
        method: T,
        with_log: bool,
    ) -> Result<R, ClientError> {
        let request_body = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: method.method_name().to_string(),
            params: method.params(),
        };
        if with_log {
            println!(
                "Request Body: {}",
                serde_json::to_string_pretty(&request_body).unwrap()
            );
        }
        let raw_response = self
            .client
            .post(&self.base_url)
            .json(&request_body)
            .send()
            .await?
            .text()
            .await?;

        if with_log {
            match serde_json::from_str::<serde_json::Value>(&raw_response) {
                Ok(parsed) => println!(
                    "Response: {}",
                    serde_json::to_string_pretty(&parsed).unwrap()
                ),
                Err(_) => println!("Raw Response: {}", raw_response),
            }
        }
        let response: RpcResponse<R> = serde_json::from_str(&raw_response)?;

        Ok(response.result)
    }

    pub async fn get_current_height(&self) -> Result<u64> {
        let response = self.send_request(GetBlockCount).await?;
        Ok(response)
    }

    pub async fn get_block(&self, height: u64) -> Result<BlockResult> {
        let response = self
            .send_request(GetBlock {
                block_height: height,
                verbosity: 1,
            })
            .await?;
        Ok(response)
    }

    pub async fn get_application_log<T: serde::de::DeserializeOwned>(
        &self,
        hash: &str,
    ) -> Result<T> {
        let app_log = self
            .send_request(GetApplicationLog {
                hash: hash.to_string(),
            })
            .await?;
        Ok(app_log)
    }

    pub async fn fetch_full_block(&self, height: u64) -> Result<(BlockResult, BlockAppLogResult)> {
        let block = self.get_block(height).await?;
        let block_app_log: BlockAppLogResult = self.get_application_log(&block.hash).await?;

        Ok((block, block_app_log))
    }

    pub async fn fetch_full_transaction(
        &self,
        tx: TransactionResult,
    ) -> Result<(TransactionResult, TransactionAppLogResult)> {
        let tx_app_log: TransactionAppLogResult = self.get_application_log(&tx.hash).await?;

        Ok((tx, tx_app_log))
    }

    pub async fn invoke_function_historic(
        &self,
        state_root_or_block: u64,
        script_hash: String,
        operation: String,
        args: Vec<NeoParam>,
    ) -> Result<Execution, ClientError> {
        let result = self
            .send_request(InvokeFunctionHistoric {
                state_root_or_block,
                script_hash,
                operation,
                args,
            })
            .await?;

        Ok(result)
    }

    pub async fn get_balance_of_historic(
        &self,
        state_root_or_block: u64,
        script_hash: &str,
        address: &str,
    ) -> Result<Execution, ClientError> {
        let args = vec![NeoParam::Object(vec![
            ("type".to_string(), NeoParam::String("Hash160".to_string())),
            ("value".to_string(), NeoParam::String(address.to_string())),
        ])];

        self.invoke_function_historic(
            state_root_or_block,
            script_hash.to_string(),
            "balanceOf".to_string(),
            args,
        )
        .await
    }

    pub async fn get_balance_of(
        &self,
        script_hash: &str,
        address: &str,
    ) -> Result<serde_json::Value, ClientError> {
        let args = vec![NeoParam::Object(vec![
            ("type".to_string(), NeoParam::String("Hash160".to_string())),
            ("value".to_string(), NeoParam::String(address.to_string())),
        ])];

        let response = self
            .send_request(InvokeFunction {
                script_hash: script_hash.to_string(),
                operation: "balanceOf".to_string(),
                args: args,
            })
            .await?;

        Ok(response)
    }
}
