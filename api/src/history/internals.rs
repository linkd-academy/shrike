use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

use crate::error::Error;
use crate::history::models::{DailyAddressBalance, DailyContractUsage, DailyTokenPrice};

pub fn list_history_balance_internal(
    conn: &PooledConnection<SqliteConnectionManager>,
    address: String,
    token: String,
    page: u32,
    per_page: u32,
    sort_by: Option<&str>,
    order: Option<&str>,
    date_init: String,
    date_end: String,
) -> Result<Vec<DailyAddressBalance>, Error> {
    let order_clause = if let (Some(sort_by), Some(order)) = (sort_by, order) {
        let valid_columns = vec!["id", "date"];
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
        "SELECT * FROM daily_address_balances WHERE address = ? AND token_contract = ? AND date BETWEEN ? AND ? {} LIMIT ? OFFSET ?",
        order_clause
    );

    let mut stmt = conn.prepare(sql.as_str()).unwrap();

    let mut rows = stmt
        .query(params![
            address,
            token,
            date_init,
            date_end,
            per_page,
            page * per_page
        ])
        .unwrap();
    let mut daily_balances = Vec::new();

    while let Some(row) = rows.next().unwrap() {
        daily_balances.push(DailyAddressBalance {
            timestamp: 0,
            block_index: row.get(1).unwrap(),
            date: row.get(2).unwrap(),
            address: row.get(3).unwrap(),
            token_contract: row.get(4).unwrap(),
            balance: row.get(5).unwrap(),
        })
    }

    if daily_balances.is_empty() {
        Err(Error {
            error: "No balances for that address/token.".to_string(),
        })
    } else {
        Ok(daily_balances)
    }
}

pub fn count_history_balance_internal(
    conn: &PooledConnection<SqliteConnectionManager>,
    address: String,
    token: String,
    date_init: String,
    date_end: String,
) -> usize {
    let sql = "
        SELECT COUNT(*) 
        FROM daily_address_balances 
        WHERE address = ? AND token_contract = ? AND date BETWEEN ? AND ?";

    conn.query_row(sql, params![address, token, date_init, date_end], |row| {
        row.get::<_, usize>(0)
    })
    .unwrap_or(0)
}

pub fn list_history_price_token_internal(
    conn: &PooledConnection<SqliteConnectionManager>,
    token: String,
    page: u32,
    per_page: u32,
    sort_by: Option<&str>,
    order: Option<&str>,
    date_init: String,
    date_end: String,
) -> Result<Vec<DailyTokenPrice>, Error> {
    let order_clause = if let (Some(sort_by), Some(order)) = (sort_by, order) {
        let valid_columns = vec!["id", "date"];
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
        "SELECT * FROM daily_token_price_history WHERE token_contract = ? AND date BETWEEN ? AND ? {} LIMIT ? OFFSET ?",
        order_clause
    );

    let mut stmt = conn.prepare(sql.as_str()).unwrap();

    let mut rows = stmt
        .query(params![
            token,
            date_init,
            date_end,
            per_page,
            page * per_page
        ])
        .unwrap();
    let mut daily_token_price = Vec::new();

    while let Some(row) = rows.next().unwrap() {
        daily_token_price.push(DailyTokenPrice {
            block_index: row.get(1).unwrap(),
            date: row.get(2).unwrap(),
            token_contract: row.get(3).unwrap(),
            price: row.get(4).unwrap(),
        })
    }

    if daily_token_price.is_empty() {
        Err(Error {
            error: "No price for that token.".to_string(),
        })
    } else {
        Ok(daily_token_price)
    }
}

pub fn count_history_price_token_internal(
    conn: &PooledConnection<SqliteConnectionManager>,
    token: String,
    date_init: String,
    date_end: String,
) -> usize {
    let sql = "
        SELECT COUNT(*)
        FROM daily_token_price_history
        WHERE token_contract = ? AND date BETWEEN ? AND ?
    ";

    conn.query_row(sql, params![token, date_init, date_end], |row| {
        row.get::<_, usize>(0)
    })
    .unwrap_or(0)
}

pub fn list_daily_contract_usage_internal(
    conn: &PooledConnection<SqliteConnectionManager>,
    contract: String,
    page: u32,
    per_page: u32,
    sort_by: Option<&str>,
    order: Option<&str>,
    date_init: String,
    date_end: String,
) -> Result<Vec<DailyContractUsage>, Error> {
    let order_clause = if let (Some(sort_by), Some(order)) = (sort_by, order) {
        let valid_columns = vec!["id", "date"];
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
        "SELECT * FROM daily_contract_usage WHERE contract = ? AND date BETWEEN ? AND ? {} LIMIT ? OFFSET ?",
        order_clause
    );

    let mut stmt = conn.prepare(sql.as_str()).unwrap();

    let mut rows = stmt
        .query(params![
            contract,
            date_init,
            date_end,
            per_page,
            page * per_page
        ])
        .unwrap();
    let mut daily_contract_usage = Vec::new();

    while let Some(row) = rows.next().unwrap() {
        daily_contract_usage.push(DailyContractUsage {
            date: row.get(1).unwrap(),
            contract: row.get(2).unwrap(),
            usage: row.get(3).unwrap(),
        })
    }

    if daily_contract_usage.is_empty() {
        Err(Error {
            error: "No contract usage for that token.".to_string(),
        })
    } else {
        Ok(daily_contract_usage)
    }
}

pub fn count_daily_contract_usage_internal(
    conn: &PooledConnection<SqliteConnectionManager>,
    contract: String,
    date_init: String,
    date_end: String,
) -> usize {
    let sql = "
        SELECT COUNT(*)
        FROM daily_contract_usage
        WHERE contract = ? AND date BETWEEN ? AND ?
    ";

    conn.query_row(sql, params![contract, date_init, date_end], |row| {
        row.get::<_, usize>(0)
    })
    .unwrap_or(0)
}
