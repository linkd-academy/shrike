use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

use crate::block::internals;
use crate::block::models::Witness;
use crate::error::Error;
use crate::shared::events;
use crate::shared::neo;
use crate::transaction::models::{Notification, State, StateValue, Transaction, TxDataList};

pub fn get_transaction_internal(
    conn: &PooledConnection<SqliteConnectionManager>,
    hash: String,
) -> Result<Transaction, Error> {
    let sql = "SELECT * FROM transactions WHERE hash = ?";
    let mut stmt = conn.prepare(sql).map_err(|err| Error {
        error: format!("Failed to prepare transaction query: {}", err),
    })?;

    let transaction_result = stmt
        .query_row([hash], |row| {
            Ok(Transaction {
                timestamp: 0, // Ajustar se necessário
                index: row.get(0)?,
                hash: row.get(1)?,
                block_index: row.get(2)?,
                vm_state: row.get(3)?,
                size: row.get(4)?,
                version: row.get(5)?,
                nonce: row.get(6)?,
                sender: row.get(7)?,
                sysfee: row.get(8)?,
                netfee: row.get(9)?,
                valid_until: row.get(10)?,
                signers: row.get(11)?,
                script: row.get(12)?,
                stack_result: row.get(13)?,
                witnesses: Vec::new(),
                notifications: Vec::new(),
            })
        })
        .map_err(|err| Error {
            error: format!("Transaction does not exist: {}", err),
        })?;

    let mut transaction = transaction_result;

    let witness_sql = "SELECT invocation, verification FROM witnesses WHERE transaction_id = ?";
    let mut stmt_witness = conn.prepare(witness_sql).map_err(|err| Error {
        error: format!("Failed to prepare witness query: {}", err),
    })?;

    let witness_iter = stmt_witness
        .query_map([transaction.index], |row| {
            Ok(Witness {
                invocation: row.get(0)?,
                verification: row.get(1)?,
            })
        })
        .map_err(|err| Error {
            error: format!("Failed to query witnesses: {}", err),
        })?;

    transaction.witnesses = witness_iter.filter_map(|witness| witness.ok()).collect();

    Ok(transaction)
}

pub fn get_transaction_notifications(
    conn: &PooledConnection<SqliteConnectionManager>,
    transaction_hash: String,
) -> Result<Vec<Notification>, Error> {
    let sql = "
        SELECT *
        FROM transaction_notifications
        WHERE transaction_hash = ?";

    let mut stmt = conn.prepare(sql).unwrap();
    let mut rows = stmt.query(params![transaction_hash]).unwrap();

    let mut notifications = Vec::new();

    while let Some(row) = rows.next().unwrap() {
        notifications.push(Notification {
            id: row.get(0).unwrap(),
            contract: row.get(2).unwrap(),
            eventname: row.get(3).unwrap(),
            state: State {
                _type: row.get(4).unwrap(),
                value: Vec::new(),
            },
        });
    }

    Ok(notifications)
}

pub fn get_notification_state_values(
    conn: &PooledConnection<SqliteConnectionManager>,
    notification_id: u64,
) -> Result<Vec<StateValue>, Error> {
    let sql = "
        SELECT type, value
        FROM transaction_notification_state_values
        WHERE transaction_notification_id = ?";

    let mut stmt = conn.prepare(sql).unwrap();
    let mut rows = stmt.query([notification_id]).unwrap();
    let mut state_values = Vec::new();

    while let Some(row) = rows.next().unwrap() {
        let state_type: String = row.get(0).unwrap();
        let state_value: Option<String> = row.get(1).ok();

        let value = state_value.map(|v| serde_json::Value::String(v));

        state_values.push(StateValue {
            _type: state_type,
            value,
        });
    }

    Ok(state_values)
}

pub fn get_sender_transactions_internal(
    conn: &PooledConnection<SqliteConnectionManager>,
    address: String,
    page: u32,
    per_page: u32,
    sort_by: Option<&str>,
    order: Option<&str>,
) -> Result<Vec<Transaction>, Error> {
    let order_clause = if let (Some(sort_by), Some(order)) = (sort_by, order) {
        let valid_columns = vec!["id"];
        if valid_columns.contains(&sort_by) {
            format!("ORDER BY {} {}", sort_by, order)
        } else {
            return Err(Error {
                error: format!("Invalid sort_by parameter: {}", sort_by),
            });
        }
    } else {
        String::new()
    };

    let sql = format!(
        "SELECT *
        FROM transactions 
        WHERE sender = ? 
        {}
        LIMIT ? OFFSET ?",
        order_clause
    );
    let mut stmt = conn.prepare(sql.as_str()).unwrap();

    let mut rows = stmt
        .query(params![address, per_page, page * per_page])
        .unwrap();

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

    match transactions.is_empty() {
        false => Ok(transactions),
        true => Err(Error {
            error: "No transactions for that sender.".to_string(),
        }),
    }
}

pub fn count_sender_transactions_internal(
    conn: &PooledConnection<SqliteConnectionManager>,
    address: String,
) -> usize {
    let sql = "
        SELECT COUNT(*)
        FROM transactions 
        WHERE sender = ?
    ";

    conn.query_row(sql, params![address], |row| row.get::<_, usize>(0))
        .unwrap_or(0)
}

pub fn get_address_transfers_internal(
    conn: &PooledConnection<SqliteConnectionManager>,
    address: String,
    page: u32,
    per_page: u32,
    sort_by: Option<&str>,
    order: Option<&str>,
) -> Result<TxDataList, Error> {
    let order_clause = if let (Some(sort_by), Some(order)) = (sort_by, order) {
        let valid_columns = vec!["id"];
        if valid_columns.contains(&sort_by) {
            format!("ORDER BY {} {}", sort_by, order)
        } else {
            return Err(Error {
                error: format!("Invalid sort_by parameter: {}", sort_by),
            });
        }
    } else {
        String::new()
    };

    let sql = format!(
        "SELECT t.*
        FROM transactions t
        INNER JOIN transaction_notifications tn ON tn.transaction_hash = t.hash
        INNER JOIN transaction_notification_state_values nsv ON tn.id = nsv.transaction_notification_id
        WHERE nsv.value = ? 
        GROUP BY t.hash
        {} LIMIT ? OFFSET ?",
        order_clause
    );

    let base64 = neo::address_to_base64(&address);
    let mut stmt = conn.prepare(sql.as_str()).unwrap();

    let mut rows = stmt
        .query(params![base64, per_page, page * per_page])
        .unwrap();

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

    let mut tx_list = TxDataList {
        address: address.clone(),
        as_sender: Vec::new(),
        as_participant: Vec::new(),
    };

    for transaction in transactions {
        let sender = transaction.clone().sender;
        let block_time =
            internals::get_block_time(conn, transaction.block_index.to_string()).unwrap();

        let notifications = get_transaction_notifications(conn, transaction.hash.clone())
            .unwrap_or_else(|_| Vec::new());

        let enriched_notifications: Vec<Notification> = notifications
            .into_iter()
            .map(|mut notification| {
                notification.state.value =
                    get_notification_state_values(conn, notification.id.unwrap())
                        .unwrap_or_else(|_| Vec::new());
                notification
            })
            .collect();

        let mut transaction_with_notifications = transaction.clone();
        transaction_with_notifications.notifications = enriched_notifications;

        let mut tx_data = events::get_transfer_events(transaction_with_notifications);
        tx_data.time = block_time;

        if sender == address {
            tx_list.as_sender.push(tx_data);
        } else {
            tx_list.as_participant.push(tx_data);
        }
    }

    if tx_list.as_sender.is_empty() && tx_list.as_participant.is_empty() {
        Err(Error {
            error: "No transfers for that sender.".to_string(),
        })
    } else {
        Ok(tx_list)
    }
}

pub fn count_address_transfers_internal(
    conn: &PooledConnection<SqliteConnectionManager>,
    address: String,
) -> usize {
    let sql = "
        SELECT COUNT(DISTINCT t.hash)
        FROM transactions t
        INNER JOIN transaction_notifications tn ON tn.transaction_hash = t.hash
        INNER JOIN transaction_notification_state_values nsv ON tn.id = nsv.transaction_notification_id
        WHERE nsv.value = ?
    ";

    let base64 = neo::address_to_base64(&address);

    conn.query_row(sql, params![base64], |row| row.get::<_, usize>(0))
        .unwrap_or(0)
}
