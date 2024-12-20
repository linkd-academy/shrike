use anyhow::Context;
use futures::future::join_all;
use futures::future::try_join_all;
use log::{error, info};
use tokio::time::sleep;

use chrono::{DateTime, NaiveTime};
use std::time::{Duration, SystemTime};

use crate::indexer::config::AppConfig;
use crate::indexer::flamingo::client::FlamingoClient;
use crate::indexer::flamingo::models::FlamingoPrice;
use crate::indexer::rpc::client::Client;
use crate::indexer::rpc::database::Database;
use crate::indexer::rpc::models::{BlockResult, TransactionResult};
use crate::indexer::utils::{conversion, logger};

pub struct Indexer<'a> {
    client: Client,
    db: Database<'a>,
    config: AppConfig,
}

impl<'a> Indexer<'a> {
    pub fn new(client: Client, db: Database<'a>, config: AppConfig) -> Self {
        Self { client, db, config }
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        let current_height = self.client.get_current_height().await?;
        let stored_height = self.db.get_last_index("blocks")?;
        info!("Chain height is {}.", current_height);

        // Ensure chain height isn't lower than stored height
        if current_height < stored_height {
            error!("Chain height is lower than stored height. Exiting..");

            Ok(())
        } else {
            let start_height = stored_height + 1;
            let index_start = SystemTime::now();
            info!("Started indexing.");
            info!(
                "Start height is {}. {} blocks to process.",
                start_height,
                current_height - start_height
            );

            self.initial_sync(start_height, current_height, self.config.batch_size)
                .await?;

            let index_end = SystemTime::now();
            let index_duration = index_end.duration_since(index_start)?;
            let new_stored_height = self
                .db
                .get_last_index("blocks")
                .context("Failed to get latest stored index")?;
            info!("Indexing completed in {} ms.", index_duration.as_millis());
            info!("New stored height is {}.", new_stored_height);

            if self.config.keep_alive {
                self.continuous_sync(new_stored_height + 1, self.config.keep_alive_interval)
                    .await?;
            }

            Ok(())
        }
    }

    async fn initial_sync(
        &self,
        mut start_height: u64,
        current_height: u64,
        batch_size: u64,
    ) -> Result<(), anyhow::Error> {
        let mut count = 0;
        info!("Updating tables:");
        while start_height < current_height {
            let end_height = std::cmp::min(start_height + batch_size, current_height);

            self.sync_between(start_height, end_height)
                .await
                .context("Failed to synchronize block range")?;

            count += end_height - start_height;
            start_height = end_height;

            logger::inline_print(&format!("\rIndexed {count} block(s)."));
        }
        println!();
        Ok(())
    }

    async fn sync_between(&self, start_height: u64, end_height: u64) -> Result<(), anyhow::Error> {
        let future_blocks = (start_height..end_height).map(|i| self.client.fetch_full_block(i));
        let all_blocks = join_all(future_blocks).await;
        let all_blocks_ref = &all_blocks;

        // Have to clone to keep all_blocks unmoved for future steps
        let transactions_with_index: Vec<(TransactionResult, u64)> = all_blocks
            .iter()
            .filter_map(|result| {
                if let Ok((block, _)) = result {
                    Some(
                        block
                            .tx
                            .iter()
                            .map(move |tx| {
                                (
                                    TransactionResult {
                                        hash: tx.hash.clone(),
                                        blockhash: Some(block.hash.clone()),
                                        timestamp: block.time,
                                        size: tx.size,
                                        version: tx.version,
                                        nonce: tx.nonce,
                                        sender: tx.sender.clone(),
                                        sysfee: tx.sysfee.clone(),
                                        netfee: tx.netfee.clone(),
                                        validuntilblock: tx.validuntilblock,
                                        signers: tx.signers.clone(),
                                        script: tx.script.clone(),
                                        witnesses: tx.witnesses.clone(),
                                    },
                                    block.index,
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                } else {
                    None
                }
            })
            .flatten()
            .collect();

        let (transactions, block_indexes): (Vec<TransactionResult>, Vec<u64>) =
            transactions_with_index.into_iter().unzip();

        let future_transactions = transactions
            .into_iter()
            .map(|tx| self.client.fetch_full_transaction(tx));
        let all_transactions = join_all(future_transactions).await;

        let all_transactions_with_index =
            all_transactions.into_iter().zip(block_indexes.into_iter());

        let flamingo_min_block_number = 664000;
        let time_threshold = NaiveTime::from_hms_opt(23, 59, 40).unwrap();

        let filtered_flamingo_blocks: Vec<&BlockResult> = all_blocks_ref
            .iter()
            .filter_map(|result| result.as_ref().ok())
            .map(|(block_result, _)| block_result)
            .filter(|block| {
                if let Some(datetime) = DateTime::from_timestamp_millis(block.time as i64) {
                    let block_time: NaiveTime = datetime.time();
                    block.index > flamingo_min_block_number && block_time > time_threshold
                } else {
                    false
                }
            })
            .collect();

        let fclient = FlamingoClient::new(None);
        let flamingo_prices: Vec<Vec<FlamingoPrice>> =
            join_all(filtered_flamingo_blocks.iter().map(|block| {
                let fclient_ref = &fclient;

                async move {
                    fclient_ref
                        .get_prices_from_block(block.index)
                        // .get_prices_from_block(664000)
                        .await
                        .unwrap_or_else(|_| vec![])
                        .into_iter()
                        .map(|mut price| {
                            price.block_index = Some(block.index);
                            price.timestamp = Some(block.time as i64);
                            price
                        })
                        .collect()
                }
            }))
            .await;

        let prepped_blocks = all_blocks.into_iter().filter_map(|result| match result {
            Ok((b, a)) => Some(conversion::convert_block_result(b, &a)),
            Err(e) => {
                panic!("Error fetching or converting block: {e:?}");
            }
        });

        let prepped_tx: Vec<_> = all_transactions_with_index
            .into_iter()
            .filter_map(|(result, block_index)| match result {
                Ok((t, a)) => Some(conversion::convert_transaction_result(t, &a, block_index)),
                Err(e) => {
                    panic!("Error fetching or converting transaction: {e:?}");
                }
            })
            .collect();

        let prepped_contracts = prepped_tx.iter().flat_map(|transaction| {
            conversion::convert_contract_result(
                transaction.script.clone(),
                transaction.notifications.clone(),
                transaction.block_index,
            )
        });

        let prepped_daily_balances = try_join_all(prepped_tx.iter().map(|transaction| async {
            conversion::convert_address_result(
                transaction.notifications.clone(),
                transaction.block_index,
                transaction.timestamp,
                &self.client,
            )
            .await
        }))
        .await?
        .into_iter()
        .flatten();

        // synced rollback point
        self.db
            .insert_blocks_transactions(prepped_blocks, prepped_tx.iter().cloned())
            .context("Failed to insert data")?;

        self.db
            .insert_contracts(prepped_contracts)
            .context("Failed to insert contracts")?;

        self.db
            .persist_daily_address_balances(prepped_daily_balances)
            .context("Failed to insert daily balances")?;

        self.db
            .persist_daily_token_price_history(flamingo_prices)
            .context("Failed to insert daily token price history")?;

        Ok(())
    }

    async fn continuous_sync(&self, start_height: u64, interval: u64) -> Result<(), anyhow::Error> {
        let mut current_height = start_height;

        info!("Listening for new blocks:");
        loop {
            let new_height = self.client.get_current_height().await?;
            if new_height > current_height {
                self.sync_between(current_height, new_height).await?;

                logger::inline_print(&format!("\rCurrent synced height: {new_height}"));
                current_height = new_height;
            }
            sleep(Duration::from_secs(interval)).await;
        }
    }
}
