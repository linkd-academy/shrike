use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

use crate::error::Error;
use crate::shared::models::DailyAddressBalance;

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
