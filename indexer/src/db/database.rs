use log::info;
use rusqlite::{params, Connection, Result, ToSql};

use crate::config::AppConfig;
use crate::flamingo::models::FlamingoPrice;

use super::model::{Block, Contract, DailyAddressBalance, Transaction};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(config: &AppConfig) -> Result<Self> {
        if config.test_db {
            info!("Using test database.");
            let conn = Connection::open("shrike_test.db3")?;

            Ok(Database { conn })
        } else {
            info!("Using database at {}.", config.db_path);
            let conn = Connection::open(&config.db_path)?;

            Ok(Database { conn })
        }
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
            reward_receiver     TEXT NOT NULL,
            witnesses           TEXT NOT NULL
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
            witnesses           TEXT NOT NULL,
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
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            transaction_hash    TEXT NOT NULL,
            type                TEXT NOT NULL,
            value               TEXT NOT NULL,
            FOREIGN KEY (transaction_hash) REFERENCES transactions (hash)
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

    pub fn insert_into_block_table(&self, block: &Block) -> Result<usize> {
        let sql = "INSERT INTO blocks (
            id, hash, size, version, merkle_root, time,
            nonce, speaker, next_consensus, reward, reward_receiver, witnesses
        ) VALUES (0, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)";

        let result = self.conn.execute(
            sql,
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
                block.witnesses
            ],
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

            let date_i64 = i64::try_from(balance.date).map_err(|_| {
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

        let mut block_values = Vec::new();
        let mut block_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        for block in blocks {
            block_values.push("(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)".to_string());
            block_params.push(Box::new(block.hash));
            block_params.push(Box::new(block.size));
            block_params.push(Box::new(block.version));
            block_params.push(Box::new(block.merkle_root));
            block_params.push(Box::new(block.time));
            block_params.push(Box::new(block.nonce));
            block_params.push(Box::new(block.speaker));
            block_params.push(Box::new(block.next_consensus));
            block_params.push(Box::new(block.reward));
            block_params.push(Box::new(block.reward_receiver));
            block_params.push(Box::new(block.witnesses));
        }

        let mut tx_values = Vec::new();
        let mut tx_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let mut notification_values = Vec::new();
        let mut notification_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let mut state_values = Vec::new();
        let mut state_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        for transaction in transactions {
            let transaction_hash = transaction.hash.clone();

            tx_values.push("(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)".to_string());
            tx_params.push(Box::new(transaction_hash.clone()));
            tx_params.push(Box::new(transaction.block_index));
            tx_params.push(Box::new(transaction.vm_state));
            tx_params.push(Box::new(transaction.size));
            tx_params.push(Box::new(transaction.version));
            tx_params.push(Box::new(transaction.nonce));
            tx_params.push(Box::new(transaction.sender));
            tx_params.push(Box::new(transaction.sysfee));
            tx_params.push(Box::new(transaction.netfee));
            tx_params.push(Box::new(transaction.valid_until));
            tx_params.push(Box::new(transaction.signers));
            tx_params.push(Box::new(transaction.script));
            tx_params.push(Box::new(transaction.witnesses));
            tx_params.push(Box::new(transaction.stack_result));

            for notification in transaction.notifications.iter() {
                notification_values.push("(?, ?, ?, ?)".to_string());
                notification_params.push(Box::new(transaction_hash.clone()));
                notification_params.push(Box::new(notification.contract.clone()));
                notification_params.push(Box::new(notification.eventname.clone()));
                notification_params.push(Box::new(notification.state._type.clone()));

                for state_value in notification.state.value.iter() {
                    state_values.push("(?, ?, ?)".to_string());
                    state_params.push(Box::new(transaction_hash.clone()));
                    state_params.push(Box::new(state_value._type.clone()));
                    state_params.push(Box::new(match &state_value.value {
                        Some(serde_json::Value::String(s)) => s.clone(),
                        Some(serde_json::Value::Number(n)) => n.to_string(),
                        Some(serde_json::Value::Null) => "null".to_string(),
                        Some(_) => serde_json::to_string(&state_value.value).unwrap_or_default(),
                        None => "".to_string(),
                    }));
                }
            }
        }

        // Executa o bulk insert para blocos
        if !block_values.is_empty() {
            let block_query = format!(
                "INSERT INTO blocks (
                    hash, size, version, merkle_root, time, nonce, speaker, next_consensus, reward, reward_receiver, witnesses
                ) VALUES {}",
                block_values.join(", ")
            );

            let block_params_ref: Vec<&dyn rusqlite::ToSql> =
                block_params.iter().map(|p| p.as_ref()).collect();

            self.conn.execute(&block_query, &block_params_ref[..])?;
        }

        // Executa o bulk insert para transações
        if !tx_values.is_empty() {
            let tx_query = format!(
                "INSERT INTO transactions (
                    hash, block_index, vm_state, size, version, nonce, sender, sysfee, netfee,
                    valid_until, signers, script, witnesses, stack_result
                ) VALUES {}",
                tx_values.join(", ")
            );

            let tx_params_ref: Vec<&dyn rusqlite::ToSql> =
                tx_params.iter().map(|p| p.as_ref()).collect();

            self.conn.execute(&tx_query, &tx_params_ref[..])?;
        }

        if !notification_values.is_empty() {
            let notification_query = format!(
                "INSERT INTO transaction_notifications (
                    transaction_hash, contract, event_name, state_type
                ) VALUES {}",
                notification_values.join(", ")
            );

            let notification_params_ref: Vec<&dyn rusqlite::ToSql> =
                notification_params.iter().map(|p| p.as_ref()).collect();

            self.conn
                .execute(&notification_query, &notification_params_ref[..])?;
        }

        if !state_values.is_empty() {
            let state_query = format!(
                "INSERT INTO transaction_notification_state_values (
                    transaction_hash, type, value
                ) VALUES {}",
                state_values.join(", ")
            );

            let state_params_ref: Vec<&dyn rusqlite::ToSql> =
                state_params.iter().map(|p| p.as_ref()).collect();

            self.conn.execute(&state_query, &state_params_ref[..])?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_last_index(&self, table: &str) -> Result<u64> {
        let sql = &format!("SELECT id FROM {table} WHERE id=(SELECT max(id) FROM {table})");
        let mut stmt = self.conn.prepare(sql)?;
        let index: u64 = stmt.query_row([], |row| row.get(0))?;

        Ok(index)
    }

    #[allow(dead_code)]
    pub fn drop_table(&self, table: &str) -> Result<usize> {
        let result = self.conn.execute(&format!("DROP TABLE {table}"), [])?;

        Ok(result)
    }
}
