use actix_web::web;
use once_cell::sync::Lazy;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use tokio::task;

use std::sync::RwLock;

use crate::shared::models::GAS_PRECISION;
use crate::ConnectionPool;

use super::models::{NetworkStatistics, ShrikeStats};

pub static CURRENT_NETWORK_STATISTICS: Lazy<RwLock<NetworkStatistics>> = Lazy::new(|| {
    let s = NetworkStatistics {
        total_transactions: 0,
        total_addresses: 0,
        total_contracts: 0,
        current_week_transactions: 0,
        current_week_addresses: 0,
        current_week_contracts: 0,
    };
    RwLock::new(s)
});

pub static CURRENT_STATS: Lazy<RwLock<ShrikeStats>> = Lazy::new(|| {
    let s = ShrikeStats {
        total_blocks: 0,
        total_transactions: 0,
        total_sysfee: 0.0,
        total_transfers: 0,
        total_senders: 0,
        total_contracts: 0,
    };
    RwLock::new(s)
});

pub fn get_stat_internal<T: rusqlite::types::FromSql>(
    conn: &PooledConnection<SqliteConnectionManager>,
    sql: &str,
) -> Option<T> {
    let mut stmt = conn.prepare(sql).expect("Failed to prepare statement");
    let total: Result<T, rusqlite::Error> = stmt.query_row([], |row| row.get(0));

    match total {
        Ok(value) => Some(value),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => {
            eprintln!("Database error: {:?} in query: {}", e, sql);
            None
        }
    }
}

pub async fn set_stats_internal(pool: web::Data<ConnectionPool>) {
    let conn1 = pool.connection.clone().get().unwrap();

    let blocks = task::spawn_blocking(move || get_blocks_internal(&conn1))
        .await
        .unwrap();

    let current_block = CURRENT_STATS.read().unwrap().total_blocks;

    if blocks > current_block {
        let conn2 = pool.connection.clone().get().unwrap();
        let conn3 = pool.connection.clone().get().unwrap();
        let conn4 = pool.connection.clone().get().unwrap();
        let conn5 = pool.connection.clone().get().unwrap();
        let conn6 = pool.connection.clone().get().unwrap();
        let conn7 = pool.connection.clone().get().unwrap();
        let conn8 = pool.connection.clone().get().unwrap();
        let conn9 = pool.connection.clone().get().unwrap();
        let conn10 = pool.connection.clone().get().unwrap();

        let transactions = task::spawn_blocking(move || get_transactions_internal(&conn2));

        let sysfees = task::spawn_blocking(move || get_sysfee_internal(&conn3));

        let transfers = task::spawn_blocking(move || get_transfers_internal(&conn4));

        let senders = task::spawn_blocking(move || get_senders_internal(&conn5));

        let contracts = task::spawn_blocking(move || get_contracts_internal(&conn6));

        let current_week_contracts =
            task::spawn_blocking(move || get_contracts_current_week_internal(&conn8));

        let current_week_transactions =
            task::spawn_blocking(move || get_transactions_current_week_internal(&conn9));

        let results = tokio::join!(
            transactions,
            sysfees,
            transfers,
            senders,
            contracts,
            current_week_contracts,
            current_week_transactions,
        );

        let total_transactions = results.0.unwrap_or(0);
        let total_contracts = results.4.unwrap_or(0);

        {
            let mut w = CURRENT_STATS.write().unwrap();

            w.total_blocks = blocks;
            w.total_transactions = total_transactions;
            w.total_sysfee = results.1.unwrap_or(0.0);
            w.total_transfers = results.2.unwrap_or(0);
            w.total_senders = results.3.unwrap_or(0);
            w.total_contracts = total_contracts;
        }

        {
            let mut w = CURRENT_NETWORK_STATISTICS.write().unwrap();

            w.total_transactions = total_transactions;
            w.total_contracts = total_contracts;
            w.current_week_contracts = results.5.unwrap_or(0);
            w.current_week_transactions = results.6.unwrap_or(0);
        }
    }
    println!("Stats refreshed. Current height is {}.", blocks);
}

pub fn get_blocks_internal(conn: &PooledConnection<SqliteConnectionManager>) -> u64 {
    let sql = "SELECT COALESCE((SELECT max(id) FROM blocks), 0)";
    get_stat_internal::<u64>(conn, sql).unwrap_or(0)
}

pub fn get_transactions_internal(conn: &PooledConnection<SqliteConnectionManager>) -> u64 {
    let sql = "SELECT COALESCE((SELECT max(id) FROM transactions), 0)";
    get_stat_internal::<u64>(conn, sql).unwrap_or(0)
}

pub fn get_sysfee_internal(conn: &PooledConnection<SqliteConnectionManager>) -> f64 {
    let sql = "SELECT COALESCE(sum(sysfee), 0) FROM transactions";
    get_stat_internal::<f64>(conn, sql).unwrap_or(0.0) / GAS_PRECISION
}

pub fn get_transfers_internal(conn: &PooledConnection<SqliteConnectionManager>) -> u64 {
    let sql = "SELECT COALESCE(SUM(1), 0) AS transfer_count
        FROM transaction_notifications
        WHERE event_name = 'Transfer'";
    get_stat_internal::<u64>(conn, sql).unwrap_or(0)
}

pub fn get_senders_internal(conn: &PooledConnection<SqliteConnectionManager>) -> u64 {
    let sql = "SELECT COALESCE(COUNT(DISTINCT sender), 0) FROM transactions";
    get_stat_internal::<u64>(conn, sql).unwrap_or(0)
}

pub fn get_contracts_internal(conn: &PooledConnection<SqliteConnectionManager>) -> u64 {
    let sql = "SELECT COALESCE(COUNT(*), 0) FROM contracts";
    let native_contracts_count = 9; // fetch natives properly in future
    get_stat_internal::<u64>(conn, sql).unwrap_or(0) + native_contracts_count
}

pub fn get_contracts_current_week_internal(
    conn: &PooledConnection<SqliteConnectionManager>,
) -> u64 {
    let sql = "SELECT COALESCE(COUNT(*), 0) 
        FROM contracts 
        INNER JOIN blocks ON blocks.id = block_index 
        WHERE time >= strftime('%s', 'now', '-7 days') * 1000";
    get_stat_internal::<u64>(conn, sql).unwrap_or(0)
}

pub fn get_transactions_current_week_internal(
    conn: &PooledConnection<SqliteConnectionManager>,
) -> u64 {
    let sql = "SELECT COALESCE(COUNT(*), 0) 
        FROM transactions 
        INNER JOIN blocks ON blocks.id = block_index 
        WHERE time >= strftime('%s', 'now', '-7 days') * 1000";
    get_stat_internal::<u64>(conn, sql).unwrap_or(0)
}
