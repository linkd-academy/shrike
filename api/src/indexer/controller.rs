use actix_web::{post, web, HttpResponse, Responder};

use crate::ConnectionPool;

use crate::indexer::config::AppConfig;
use crate::indexer::rpc::client::Client as RpcClient;
use crate::indexer::rpc::database::Database as LocalDatabase;
use crate::indexer::spawn::indexer::Indexer;

pub async fn initilize_indexer_setup(pool: web::Data<ConnectionPool>) -> impl Responder {
    let conn = &pool.connection.get().unwrap();

    let db = match LocalDatabase::new(&conn) {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to initialize database"),
    };

    // make sure WAL journal mode is enabled
    if let Err(_) = db.set_to_wal() {
        return HttpResponse::InternalServerError().body("Failed to set to WAL");
    }

    // fails if it already exists
    if let Err(_) = db.create_block_table() {
        return HttpResponse::InternalServerError().body("Failed to create block table");
    }
    if let Err(_) = db.create_transaction_table() {
        return HttpResponse::InternalServerError().body("Failed to create transaction table");
    }
    if let Err(_) = db.create_transaction_notification_table() {
        return HttpResponse::InternalServerError()
            .body("Failed to create transaction notification table");
    }
    if let Err(_) = db.create_transaction_notification_state_value_table() {
        return HttpResponse::InternalServerError()
            .body("Failed to create transaction notification state value table");
    }
    if let Err(_) = db.create_daily_address_balances() {
        return HttpResponse::InternalServerError()
            .body("Failed to create daily_address_balances table");
    }
    if let Err(_) = db.create_daily_token_price_history() {
        return HttpResponse::InternalServerError()
            .body("Failed to create daily_token_price_history table");
    }
    if let Err(_) = db.create_contract_table() {
        return HttpResponse::InternalServerError().body("Failed to create contract table");
    }
    if let Err(_) = db.create_daily_contract_usage() {
        return HttpResponse::InternalServerError()
            .body("Failed to create daily contract usage table");
    }

    // create indexes if they don't exist
    if let Err(_) = db.create_index("idx_blocks_hash", "blocks", "hash") {
        return HttpResponse::InternalServerError().body("Failed to create block index");
    }
    if let Err(_) = db.create_index("idx_tx_hash", "transactions", "hash") {
        return HttpResponse::InternalServerError().body("Failed to create txid index");
    }
    if let Err(_) = db.create_index("idx_tx_senders", "transactions", "sender") {
        return HttpResponse::InternalServerError().body("Failed to create txsender index");
    }
    if let Err(_) = db.create_index("idx_transaction_block_index", "transactions", "block_index") {
        return HttpResponse::InternalServerError().body("Failed to create block index");
    }
    if let Err(_) = db.create_index(
        "idx_transaction_notifications_event_name",
        "transaction_notifications",
        "event_name",
    ) {
        return HttpResponse::InternalServerError()
            .body("Failed to create transaction_notifications event_name index");
    }
    if let Err(_) = db.create_index(
        "idx_transaction_notification_state_values_value",
        "transaction_notification_state_values",
        "value",
    ) {
        return HttpResponse::InternalServerError()
            .body("Failed to create transaction_notification_state_values value index");
    }
    if let Err(_) = db.create_index(
        "idx_daily_address_balances_address",
        "daily_address_balances",
        "address",
    ) {
        return HttpResponse::InternalServerError().body("Failed to create address index");
    }
    if let Err(_) = db.create_index(
        "idx_daily_address_balances_date",
        "daily_address_balances",
        "date",
    ) {
        return HttpResponse::InternalServerError().body("Failed to create date index");
    }
    if let Err(_) = db.create_index(
        "idx_daily_token_price_history_date",
        "daily_token_price_history",
        "date",
    ) {
        return HttpResponse::InternalServerError().body("Failed to create address index");
    }
    if let Err(_) = db.create_index("idx_contract_hash", "contracts", "hash") {
        return HttpResponse::InternalServerError().body("Failed to create contract index");
    }
    if let Err(_) = db.create_index(
        "idx_daily_contract_usage_date",
        "daily_contract_usage",
        "date",
    ) {
        return HttpResponse::InternalServerError()
            .body("Failed to create daily contract usage date index");
    }
    if let Err(_) = db.create_index(
        "idx_daily_contract_usage_contract",
        "daily_contract_usage",
        "contract",
    ) {
        return HttpResponse::InternalServerError()
            .body("Failed to create daily contract usage contract index");
    }

    HttpResponse::Ok().json(true)
}

#[post("/v1/indexer/run")]
async fn run_indexer(pool: web::Data<ConnectionPool>) -> impl Responder {
    let config = AppConfig::new();

    let client = RpcClient::new(&config);
    let conn = &pool.connection.get().unwrap();

    let db = match LocalDatabase::new(&conn) {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to initialize database"),
    };

    let indexer = Indexer::new(client, db, config);
    if let Err(_) = indexer.run().await {
        return HttpResponse::InternalServerError().json("Failed to run indexer");
    }

    HttpResponse::Ok().json(true)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(run_indexer);
}
