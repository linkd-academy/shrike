use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;

use super::models::{Block, Witness};
use crate::error::Error;
use crate::shared::checker;
use crate::transaction::models::Transaction;

pub fn get_block_internal(
    conn: &PooledConnection<SqliteConnectionManager>,
    path: String,
) -> Result<Block, Error> {
    match path.trim().parse::<u64>() {
        Ok(id) => {
            let sql = "SELECT * FROM blocks WHERE id = ?";
            let mut stmt = conn.prepare(sql).map_err(|err| Error {
                error: format!("Failed to prepare block query: {}", err),
            })?;

            let block_result = stmt
                .query_row([id], |row| {
                    Ok(Block {
                        index: row.get(0)?,
                        hash: row.get(1)?,
                        size: row.get(2)?,
                        version: row.get(3)?,
                        merkle_root: row.get(4)?,
                        time: row.get(5)?,
                        nonce: row.get(6)?,
                        speaker: row.get(7)?,
                        next_consensus: row.get(8)?,
                        reward: row.get(9)?,
                        reward_receiver: row.get(10)?,
                        witnesses: Vec::new(),
                    })
                })
                .map_err(|err| Error {
                    error: format!("Block does not exist: {}", err),
                })?;

            let mut block = block_result;

            let witness_sql =
                "SELECT invocation, verification FROM witnesses WHERE block_index = ?";
            let mut stmt_witness = conn.prepare(witness_sql).map_err(|err| Error {
                error: format!("Failed to prepare witness query: {}", err),
            })?;

            let witness_iter = stmt_witness
                .query_map([block.index], |row| {
                    Ok(Witness {
                        invocation: row.get(0)?,
                        verification: row.get(1)?,
                    })
                })
                .map_err(|err| Error {
                    error: format!("Failed to query witnesses: {}", err),
                })?;

            block.witnesses = witness_iter.filter_map(|witness| witness.ok()).collect();

            Ok(block)
        }
        Err(_) => {
            if !checker::is_neo_txid_hash(&path) {
                return Err(Error {
                    error: "Invalid block hash.".to_string(),
                });
            }

            let sql = "SELECT * FROM blocks WHERE hash = ?";
            let mut stmt = conn.prepare(sql).map_err(|err| Error {
                error: format!("Failed to prepare block query by hash: {}", err),
            })?;

            let block_result = stmt
                .query_row([path], |row| {
                    Ok(Block {
                        index: row.get(0)?,
                        hash: row.get(1)?,
                        size: row.get(2)?,
                        version: row.get(3)?,
                        merkle_root: row.get(4)?,
                        time: row.get(5)?,
                        nonce: row.get(6)?,
                        speaker: row.get(7)?,
                        next_consensus: row.get(8)?,
                        reward: row.get(9)?,
                        reward_receiver: row.get(10)?,
                        witnesses: Vec::new(),
                    })
                })
                .map_err(|err| Error {
                    error: format!("Block does not exist: {}", err),
                })?;

            let mut block = block_result;

            let witness_sql =
                "SELECT invocation, verification FROM witnesses WHERE block_index = ?";
            let mut stmt_witness = conn.prepare(witness_sql).map_err(|err| Error {
                error: format!("Failed to prepare witness query: {}", err),
            })?;

            let witness_iter = stmt_witness
                .query_map([block.index], |row| {
                    Ok(Witness {
                        invocation: row.get(0)?,
                        verification: row.get(1)?,
                    })
                })
                .map_err(|err| Error {
                    error: format!("Failed to query witnesses: {}", err),
                })?;

            block.witnesses = witness_iter.filter_map(|witness| witness.ok()).collect();

            Ok(block)
        }
    }
}

pub fn get_block_time(
    conn: &PooledConnection<SqliteConnectionManager>,
    path: String,
) -> Result<u64, Error> {
    match path.trim().parse::<u64>() {
        Ok(id) => {
            let sql = "SELECT time FROM blocks WHERE id = ?";
            let mut stmt = conn.prepare(sql).unwrap();

            let result = stmt.query_row([id], |row| row.get(0));

            result.map_err(|_| Error {
                error: "Block does not exist.".to_string(),
            })
        }
        Err(_) => {
            if !checker::is_neo_txid_hash(&path) {
                return Err(Error {
                    error: "Invalid block hash.".to_string(),
                });
            }

            let sql = "SELECT time FROM blocks WHERE hash = ?";
            let mut stmt = conn.prepare(sql).unwrap();

            let result = stmt.query_row([path], |row| row.get(0));

            result.map_err(|_| Error {
                error: "Block does not exist.".to_string(),
            })
        }
    }
}

pub fn get_block_transactions_internal(
    conn: &PooledConnection<SqliteConnectionManager>,
    path: String,
) -> Result<Vec<Transaction>, Error> {
    match path.trim().parse::<u64>() {
        Ok(id) => {
            let sql = "SELECT * FROM transactions WHERE block_index = ?";
            let mut stmt = conn.prepare(sql).unwrap();

            let mut rows = stmt.query([id]).unwrap();
            let mut transactions = Vec::new();

            while let Some(row) = rows.next().unwrap() {
                transactions.push(Transaction {
                    timestamp: 0,
                    index: row.get(0).unwrap(),
                    hash: row.get(1).unwrap(),
                    block_index: row.get(2).unwrap(),
                    vm_state: row.get(3).unwrap(),
                    size: row.get(4).unwrap(),
                    version: row.get(5).unwrap(),
                    nonce: row.get(6).unwrap(),
                    sender: row.get(7).unwrap(),
                    sysfee: row.get(8).unwrap(),
                    netfee: row.get(9).unwrap(),
                    valid_until: row.get(10).unwrap(),
                    signers: row.get(11).unwrap(),
                    script: row.get(12).unwrap(),
                    stack_result: row.get(13).unwrap(),
                    witnesses: Vec::new(),
                    notifications: Vec::new(),
                })
            }
            Ok(transactions)
        }
        Err(_) => {
            if !checker::is_neo_txid_hash(&path) {
                return Err(Error {
                    error: "Invalid block hash.".to_string(),
                });
            }

            let sql = "SELECT * FROM transactions WHERE block_index = (SELECT id FROM blocks WHERE hash = ?)";
            let mut stmt = conn.prepare(sql).unwrap();

            let mut rows = stmt.query([path]).unwrap();
            let mut transactions = Vec::new();

            while let Some(row) = rows.next().unwrap() {
                transactions.push(Transaction {
                    timestamp: 0,
                    index: row.get(0).unwrap(),
                    hash: row.get(1).unwrap(),
                    block_index: row.get(2).unwrap(),
                    vm_state: row.get(3).unwrap(),
                    size: row.get(4).unwrap(),
                    version: row.get(5).unwrap(),
                    nonce: row.get(6).unwrap(),
                    sender: row.get(7).unwrap(),
                    sysfee: row.get(8).unwrap(),
                    netfee: row.get(9).unwrap(),
                    valid_until: row.get(10).unwrap(),
                    signers: row.get(11).unwrap(),
                    script: row.get(12).unwrap(),
                    stack_result: row.get(13).unwrap(),
                    witnesses: Vec::new(),
                    notifications: Vec::new(),
                })
            }
            Ok(transactions)
        }
    }
}
