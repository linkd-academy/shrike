use actix_web::{get, web, HttpResponse, Responder};

use crate::error::Error;
use crate::ConnectionPool;

use super::internals;

#[get("/v1/block/{id}")]
async fn get_block(pool: web::Data<ConnectionPool>, path: web::Path<String>) -> impl Responder {
    let conn = &pool.connection.get().unwrap();
    let id = path.into_inner();

    let mut block = match internals::get_block_internal(conn, id) {
        Ok(blc) => blc,
        Err(err) => return HttpResponse::NotFound().json(err),
    };

    let witnesses = match internals::get_witnesses(conn, block.index.clone()) {
        Ok(witnesses) => witnesses,
        Err(err) => return HttpResponse::InternalServerError().json(err),
    };
    block.witnesses = witnesses;

    HttpResponse::Ok().json(block)
}

#[get("/v1/block/{id}/transactions")]
async fn get_block_transactions(
    pool: web::Data<ConnectionPool>,
    path: web::Path<String>,
) -> impl Responder {
    let conn = &pool.connection.get().unwrap();
    let id = path.into_inner();

    let transactions = internals::get_block_transactions_internal(conn, id).unwrap();

    match transactions.is_empty() {
        false => HttpResponse::Ok().json(transactions),
        true => HttpResponse::Ok().json(Error {
            error: "No transactions for that block.".to_string(),
        }),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_block).service(get_block_transactions);
}
