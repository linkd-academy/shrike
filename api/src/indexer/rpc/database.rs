use log::info;
use rusqlite::{params, Result, ToSql};

use crate::indexer::flamingo::models::FlamingoPrice;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;

use crate::block::models::Block;
use crate::history::models::DailyAddressBalance;
use crate::indexer::rpc::models::Contract;
use crate::transaction::models::Transaction;

pub struct Database<'a> {
    conn: &'a PooledConnection<SqliteConnectionManager>,
}

impl<'a> Database<'a> {
    pub fn new(conn: &'a PooledConnection<SqliteConnectionManager>) -> Result<Self> {
        Ok(Database { conn })
    }

    pub fn set_to_wal(&self) -> Result<()> {
        let wal_active: String = self
            .conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
        if wal_active == "wal" {
            info!("WAL mode already active.");
        } else {
            let _: String = self
                .conn
                .query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;
            info!("Set db to WAL mode.");
        }

        Ok(())
    }

    pub fn create_index(&self, name: &str, table: &str, column: &str) -> Result<usize> {
        let sql = format!("CREATE INDEX IF NOT EXISTS {name} ON {table} ({column})");
        let result = self.conn.execute(&sql, [])?;

        Ok(result)
    }

    pub fn create_block_table(&self) -> Result<usize> {
        let result = self.conn.execute(
            "CREATE TABLE IF NOT EXISTS blocks (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            hash                TEXT NOT NULL UNIQUE,
            size                INTEGER NOT NULL,
            version             INTEGER NOT NULL,
            merkle_root         TEXT NOT NULL,
            time                INTEGER NOT NULL,
            nonce               TEXT NOT NULL,
            speaker             INTEGER NOT NULL,
            next_consensus      TEXT NOT NULL,
            reward              FLOAT NOT NULL,
            reward_receiver     TEXT NOT NULL
        )",
            [],
        )?;

        Ok(result)
    }

    pub fn create_witnesses_table(&self) -> Result<usize> {
        let result = self.conn.execute(
            "CREATE TABLE IF NOT EXISTS witnesses (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            block_index         INTEGER NULL,
            transaction_id      INTEGER NULL,
            invocation          TEXT NOT NULL,
            verification        TEXT NOT NULL,
            FOREIGN KEY (block_index) REFERENCES blocks (id),
            FOREIGN KEY (transaction_id) REFERENCES transactions (id)
        )",
            [],
        )?;

        Ok(result)
    }

    pub fn create_transaction_table(&self) -> Result<usize> {
        let result = self.conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            hash                TEXT NOT NULL UNIQUE,
            block_index         INTEGER NOT NULL,
            vm_state            TEXT NOT NULL,
            size                INTEGER NOT NULL,
            version             INTEGER NOT NULL,
            nonce               INTEGER NOT NULL,
            sender              TEXT NOT NULL,
            sysfee              TEXT NOT NULL,
            netfee              TEXT NOT NULL,
            valid_until         INTEGER NOT NULL,
            signers             TEXT NOT NULL,
            script              TEXT NOT NULL,
            stack_result        TEXT,
            FOREIGN KEY (block_index) REFERENCES blocks (id)
        )",
            [],
        )?;

        Ok(result)
    }

    pub fn create_transaction_notification_table(&self) -> Result<usize> {
        let result = self.conn.execute(
            "CREATE TABLE IF NOT EXISTS transaction_notifications (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            transaction_hash    TEXT NOT NULL,
            contract            TEXT NOT NULL,
            event_name          TEXT NOT NULL,
            state_type          TEXT NOT NULL,
            FOREIGN KEY (transaction_hash) REFERENCES transactions (hash)
        )",
            [],
        )?;

        Ok(result)
    }
    pub fn create_transaction_notification_state_value_table(&self) -> Result<usize> {
        let result = self.conn.execute(
            "CREATE TABLE IF NOT EXISTS transaction_notification_state_values (
            id                              INTEGER PRIMARY KEY AUTOINCREMENT,
            transaction_notification_id     INTEGER NOT NULL,
            type                            TEXT NOT NULL,
            value                           TEXT NULL,
            FOREIGN KEY (transaction_notification_id) REFERENCES transaction_notifications (id)
        )",
            [],
        )?;

        Ok(result)
    }

    pub fn create_daily_address_balances(&self) -> Result<usize> {
        let result = self.conn.execute(
            "CREATE TABLE IF NOT EXISTS daily_address_balances (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            block_index         INTEGER NOT NULL,
            date                TEXT NOT NULL,    
            address             TEXT NOT NULL,
            token_contract      TEXT NOT NULL,    
            balance             INTEGER NOT NULL,
            UNIQUE (date, address, token_contract), 
            FOREIGN KEY (block_index) REFERENCES blocks (id)
        )",
            [],
        )?;

        Ok(result)
    }

    pub fn create_daily_token_price_history(&self) -> Result<usize> {
        let result = self.conn.execute(
            "CREATE TABLE IF NOT EXISTS daily_token_price_history (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            block_index         INTEGER NOT NULL,
            date                TEXT NOT NULL,    
            token_contract      TEXT NOT NULL,    
            price               FLOAT NOT NULL,
            UNIQUE (date, token_contract), 
            FOREIGN KEY (block_index) REFERENCES blocks (id)
        )",
            [],
        )?;

        Ok(result)
    }

    pub fn create_contract_table(&self) -> Result<usize> {
        let result = self.conn.execute(
            "CREATE TABLE IF NOT EXISTS contracts (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            block_index         INTEGER NOT NULL,
            hash                TEXT NOT NULL UNIQUE,
            contract_type       TEXT NOT NULL,
            FOREIGN KEY (block_index) REFERENCES blocks (id)
        )",
            [],
        )?;

        Ok(result)
    }

    pub fn create_daily_contract_usage(&self) -> Result<usize> {
        let result = self.conn.execute(
            "CREATE TABLE IF NOT EXISTS daily_contract_usage (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            date                TEXT NOT NULL,    
            contract            TEXT NOT NULL,    
            usage               INTEGER NOT NULL,
            UNIQUE (date, contract) 
        )",
            [],
        )?;

        Ok(result)
    }

    pub fn insert_contracts(&self, contracts: impl Iterator<Item = Contract>) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;

        let mut values: Vec<String> = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        for contract in contracts {
            values.push("(?, ?, ?)".to_string());
            params.push(Box::new(contract.block_index));
            params.push(Box::new(contract.hash));
            params.push(Box::new(contract.contract_type));
        }

        if !values.is_empty() {
            let query = format!(
                "INSERT INTO contracts (block_index, hash, contract_type) VALUES {}",
                values.join(", ")
            );

            let params_ref: Vec<&dyn rusqlite::ToSql> = params.iter().map(|v| v.as_ref()).collect();

            self.conn.execute(&query, &params_ref[..])?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn persist_daily_address_balances(
        &self,
        balances: impl Iterator<Item = DailyAddressBalance>,
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;

        let mut values: Vec<String> = Vec::new();
        let mut params: Vec<Box<dyn ToSql>> = Vec::new();

        for balance in balances {
            values.push("(strftime('%Y-%m-%d', ? / 1000, 'unixepoch'), ?, ?, ?, ?)".to_string());

            let date_i64 = i64::try_from(balance.timestamp).map_err(|_| {
                rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed to convert u64 to i64",
                )))
            })?;

            params.push(Box::new(date_i64));
            params.push(Box::new(balance.address.clone()));
            params.push(Box::new(balance.token_contract.clone()));
            params.push(Box::new(balance.balance));
            params.push(Box::new(balance.block_index));
        }

        if !values.is_empty() {
            let query = format!(
                "INSERT INTO daily_address_balances (
                    date, address, token_contract, balance, block_index
                ) VALUES {} 
                ON CONFLICT (date, address, token_contract) 
                DO UPDATE SET balance = excluded.balance, block_index = excluded.block_index",
                values.join(", ")
            );

            let params_ref: Vec<&dyn ToSql> = params.iter().map(|v| v.as_ref()).collect();

            self.conn.execute(&query, &params_ref[..])?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn persist_daily_token_price_history(
        &self,
        prices_vec: Vec<Vec<FlamingoPrice>>,
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;

        let mut values: Vec<String> = Vec::new();
        let mut params: Vec<Box<dyn ToSql>> = Vec::new();

        for prices in prices_vec {
            for price in prices {
                values.push("(strftime('%Y-%m-%d', ? / 1000, 'unixepoch'), ?, ?, ?)".to_string());

                params.push(Box::new(price.timestamp));
                params.push(Box::new(price.hash));
                params.push(Box::new(price.usd_price));
                params.push(Box::new(price.block_index));
            }
        }

        if !values.is_empty() {
            let query = format!(
                "INSERT INTO daily_token_price_history (
                    date, token_contract, price, block_index
                ) VALUES {} 
                ON CONFLICT (date, token_contract) 
                DO UPDATE SET price = excluded.price, block_index = excluded.block_index",
                values.join(", ")
            );

            let params_ref: Vec<&dyn ToSql> = params.iter().map(|v| v.as_ref()).collect();

            self.conn.execute(&query, &params_ref[..])?;
        }

        tx.commit()?;
        Ok(())
    }

    // synced rollback for both tables
    pub fn insert_blocks_transactions(
        &self,
        blocks: impl Iterator<Item = Block>,
        transactions: impl Iterator<Item = Transaction>,
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;

        let block_query = "
            INSERT INTO blocks (
                hash, size, version, merkle_root, time, nonce, speaker, next_consensus, reward, reward_receiver
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id";
        let mut stmt_block = self.conn.prepare(block_query)?;

        let witness_query = "
            INSERT INTO witnesses (
                block_index, transaction_id, invocation, verification
            ) VALUES (?, ?, ?, ?)";
        let mut stmt_witness = self.conn.prepare(witness_query)?;

        let transaction_query = "
            INSERT INTO transactions (
                hash, block_index, vm_state, size, version, nonce, sender, sysfee, netfee,
                valid_until, signers, script, stack_result
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id";
        let mut stmt_transaction = self.conn.prepare(transaction_query)?;

        let notification_query = "
            INSERT INTO transaction_notifications (
                transaction_hash, contract, event_name, state_type
            ) VALUES (?, ?, ?, ?) RETURNING id";
        let mut stmt_notification = self.conn.prepare(notification_query)?;

        let daily_contract_usage_query = "
            INSERT INTO daily_contract_usage (date, contract, usage)
            VALUES (strftime('%Y-%m-%d', ? / 1000, 'unixepoch'), ?, 1)
            ON CONFLICT(date, contract)
            DO UPDATE SET usage = usage + 1";
        let mut stmt_daily_contract_usage = self.conn.prepare(daily_contract_usage_query)?;

        let state_query = "
            INSERT INTO transaction_notification_state_values (
                transaction_notification_id, type, value
            ) VALUES (?, ?, ?)";
        let mut stmt_state = self.conn.prepare(state_query)?;

        for block in blocks {
            let block_id: i64 = stmt_block.query_row(
                params![
                    block.hash,
                    block.size,
                    block.version,
                    block.merkle_root,
                    block.time,
                    block.nonce,
                    block.speaker,
                    block.next_consensus,
                    block.reward,
                    block.reward_receiver,
                ],
                |row| row.get(0),
            )?;

            for witness in block.witnesses {
                stmt_witness.execute(params![
                    block_id,
                    None::<i64>,
                    witness.invocation,
                    witness.verification,
                ])?;
            }
        }

        for transaction in transactions {
            let transaction_id: i64 = stmt_transaction.query_row(
                params![
                    transaction.hash,
                    transaction.block_index,
                    transaction.vm_state,
                    transaction.size,
                    transaction.version,
                    transaction.nonce,
                    transaction.sender,
                    transaction.sysfee,
                    transaction.netfee,
                    transaction.valid_until,
                    transaction.signers,
                    transaction.script,
                    transaction.stack_result,
                ],
                |row| row.get(0),
            )?;

            for witness in transaction.witnesses {
                stmt_witness.execute(params![
                    None::<i64>,
                    transaction_id,
                    witness.invocation,
                    witness.verification,
                ])?;
            }

            for notification in transaction.notifications.iter() {
                let notification_id: i64 = stmt_notification.query_row(
                    params![
                        transaction.hash,
                        notification.contract,
                        notification.eventname,
                        notification.state._type,
                    ],
                    |row| row.get(0),
                )?;

                stmt_daily_contract_usage
                    .execute(params![transaction.timestamp, notification.contract])?;

                for state_value in notification.state.value.iter() {
                    stmt_state.execute(params![
                        notification_id,
                        state_value._type,
                        match &state_value.value {
                            Some(serde_json::Value::String(s)) => Some(s.clone()),
                            Some(serde_json::Value::Number(n)) => Some(n.to_string()),
                            Some(serde_json::Value::Null) => None,
                            None => None,
                            _ =>
                                Some(serde_json::to_string(&state_value.value).unwrap_or_default()),
                        }
                    ])?;
                }
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_last_index(&self, table: &str) -> Result<u64> {
        let sql = format!("SELECT id FROM {table} WHERE id=(SELECT max(id) FROM {table})");
        let mut stmt = self.conn.prepare(&sql)?;

        let index: u64 = stmt.query_row([], |row| row.get(0)).unwrap_or(0);

        Ok(index)
    }

    #[allow(dead_code)]
    pub fn drop_table(&self, table: &str) -> Result<usize> {
        let result = self.conn.execute(&format!("DROP TABLE {table}"), [])?;

        Ok(result)
    }
}
