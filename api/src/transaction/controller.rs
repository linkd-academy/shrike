use actix_web::{get, web, HttpResponse, Responder};

use crate::error::Error;
use crate::shared::checker;
use crate::shared::models::PaginationAndFilterParams;
use crate::shared::utils::normalize_pagination;
use crate::ConnectionPool;

use super::internals;

#[get("/v1/transaction/{hash}")]
async fn get_transaction(
    pool: web::Data<ConnectionPool>,
    path: web::Path<String>,
) -> impl Responder {
    let hash = path.into_inner();

    if !checker::is_neo_txid_hash(&hash) {
        return HttpResponse::Ok().json(Error {
            error: "Invalid transaction hash.".to_string(),
        });
    }

    let conn = &pool.connection.get().unwrap();
    let mut transaction = match internals::get_transaction_internal(conn, hash.clone()) {
        Ok(tx) => tx,
        Err(err) => return HttpResponse::NotFound().json(err),
    };

    let notifications = match internals::get_transaction_notifications(conn, hash.clone()) {
        Ok(notifs) => notifs,
        Err(err) => return HttpResponse::InternalServerError().json(err),
    };

    let mut enriched_notifications = Vec::new();
    for mut notification in notifications {
        let state_values =
            match internals::get_notification_state_values(conn, notification.id.unwrap()) {
                Ok(states) => states,
                Err(err) => return HttpResponse::InternalServerError().json(err),
            };
        notification.state.value = state_values;
        enriched_notifications.push(notification);
    }

    transaction.notifications = enriched_notifications;

    HttpResponse::Ok().json(transaction)
}

#[get("/v1/transaction/sender/{address}")]
async fn get_sender_transactions(
    pool: web::Data<ConnectionPool>,
    path: web::Path<String>,
    query_parameter: web::Query<PaginationAndFilterParams>,
) -> impl Responder {
    let address = path.into_inner();

    if !checker::is_neo_address(&address) {
        return HttpResponse::Ok().json(Error {
            error: "Invalid address.".to_string(),
        });
    }

    let (page, per_page, sort_by, order) = match normalize_pagination(&query_parameter) {
        Ok(result) => result,
        Err(response) => return response,
    };

    let conn = &pool.connection.get().unwrap();

    let transactions = internals::get_sender_transactions_internal(
        conn,
        address,
        page,
        per_page,
        sort_by.as_deref(),
        order.as_deref(),
    );

    match transactions {
        Ok(txs) => HttpResponse::Ok().json(txs),
        Err(err) => HttpResponse::Ok().json(err),
    }
}

#[get("/v1/transaction/transfers/{address}")]
async fn get_address_transfers(
    pool: web::Data<ConnectionPool>,
    path: web::Path<String>,
    query_parameter: web::Query<PaginationAndFilterParams>,
) -> impl Responder {
    let address = path.into_inner();

    if !checker::is_neo_address(&address) {
        return HttpResponse::Ok().json(Error {
            error: "Invalid address.".to_string(),
        });
    }

    let (page, per_page, sort_by, order) = match normalize_pagination(&query_parameter) {
        Ok(result) => result,
        Err(response) => return response,
    };

    let conn = &pool.connection.get().unwrap();

    let transfer_list = internals::get_address_transfers_internal(
        conn,
        address,
        page,
        per_page,
        sort_by.as_deref(),
        order.as_deref(),
    );

    match transfer_list {
        Ok(txs) => HttpResponse::Ok().json(txs),
        Err(err) => HttpResponse::Ok().json(err),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_transaction)
        .service(get_sender_transactions)
        .service(get_address_transfers);
}
