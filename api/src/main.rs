mod block;
mod error;
mod history;
mod indexer;
mod shared;
mod stat;
mod transaction;

use crate::indexer::controller::initilize_indexer_setup;
use crate::shared::config::Config;
use crate::shared::db::DB_PATH;
use actix_cors::Cors;
use actix_web::{http::header, web, App, HttpServer};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use rusqlite::OpenFlags;
use tokio::{task, time};

use std::time::Duration;

const REFRESH_INTERVAL: u64 = 3; // how often we check for a new block and refresh stats in seconds

pub struct ConnectionPool {
    connection: Pool<SqliteConnectionManager>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::new().expect("Failed to load configuration");

    let db_path = DB_PATH
        .to_str()
        .expect("Failed to convert database path to str");

    let _ = Connection::open(db_path);

    let manager_ro =
        SqliteConnectionManager::file(db_path).with_flags(OpenFlags::SQLITE_OPEN_READ_ONLY);
    let pool_ro = Pool::new(manager_ro).unwrap();
    let connection_pool_ro = web::Data::new(ConnectionPool {
        connection: pool_ro,
    });
    let internal_connection_ro = connection_pool_ro.clone();

    let manager_rw =
        SqliteConnectionManager::file(db_path).with_flags(OpenFlags::SQLITE_OPEN_READ_WRITE);
    let pool_rw = Pool::new(manager_rw).unwrap();
    let connection_pool_rw = web::Data::new(ConnectionPool {
        connection: pool_rw,
    });

    initilize_indexer_setup(connection_pool_rw.clone()).await;

    task::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(REFRESH_INTERVAL));
        loop {
            let c = internal_connection_ro.clone();
            interval.tick().await;
            stat::internals::set_stats_internal(c).await;
        }
    });

    println!("Opening to requests on http://0.0.0.0:{}.", config.api_port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET"])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600);
        App::new()
            .wrap(cors)
            .app_data(connection_pool_ro.clone())
            .configure(block::controller::config)
            .configure(transaction::controller::config)
            .configure(stat::controller::config)
            .configure(history::controller::config)
            .app_data(connection_pool_rw.clone())
            .configure(indexer::controller::config)
    })
    .bind(("0.0.0.0", config.api_port))?
    .run()
    .await
}
