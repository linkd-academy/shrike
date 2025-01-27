use actix_web::{get, web, HttpResponse, Responder};

use crate::error::Error;
use crate::shared::checker;
use crate::shared::models::{PagedResp, PaginationAndFilterParams};
use crate::shared::utils::{normalize_filter, normalize_pagination};
use crate::ConnectionPool;

use super::internals;

#[get("/v1/balance-history/{address}/{token}")]
async fn list_balance_history(
    pool: web::Data<ConnectionPool>,
    path: web::Path<(String, String)>,
    query_parameter: web::Query<PaginationAndFilterParams>,
) -> impl Responder {
    let (address, token) = path.into_inner();

    if !checker::is_neo_address(&address) {
        return HttpResponse::Ok().json(Error {
            error: "Invalid address.".to_string(),
        });
    }

    let (page, per_page, sort_by, order) = match normalize_pagination(&query_parameter) {
        Ok(result) => result,
        Err(response) => return response,
    };

    let (date_init, date_end) = match normalize_filter(&query_parameter) {
        Ok(result) => result,
        Err(response) => return response,
    };

    let conn = &pool.connection.get().unwrap();
    let balance_history = internals::list_history_balance_internal(
        conn,
        address.clone(),
        token.clone(),
        page,
        per_page,
        sort_by.as_deref(),
        order.as_deref(),
        date_init.clone(),
        date_end.clone(),
    );

    match balance_history {
        Ok(bh) => HttpResponse::Ok().json(PagedResp::new(
            bh,
            internals::count_history_balance_internal(
                conn,
                address.clone(),
                token.clone(),
                date_init.clone(),
                date_end.clone(),
            ),
        )),
        Err(err) => HttpResponse::Ok().json(err),
    }
}

#[get("/v1/tokens/{token}/price-history")]
async fn list_token_price_history(
    pool: web::Data<ConnectionPool>,
    path: web::Path<String>,
    query_parameter: web::Query<PaginationAndFilterParams>,
) -> impl Responder {
    let token = path.into_inner();

    let (page, per_page, sort_by, order) = match normalize_pagination(&query_parameter) {
        Ok(result) => result,
        Err(response) => return response,
    };

    let (date_init, date_end) = match normalize_filter(&query_parameter) {
        Ok(result) => result,
        Err(response) => return response,
    };

    let conn = &pool.connection.get().unwrap();
    let price_history = internals::list_history_price_token_internal(
        conn,
        token.clone(),
        page,
        per_page,
        sort_by.as_deref(),
        order.as_deref(),
        date_init.clone(),
        date_end.clone(),
    );

    match price_history {
        Ok(ph) => HttpResponse::Ok().json(PagedResp::new(
            ph,
            internals::count_history_price_token_internal(
                conn,
                token.clone(),
                date_init.clone(),
                date_end.clone(),
            ),
        )),
        Err(err) => HttpResponse::Ok().json(err),
    }
}

#[get("/v1/contracts/{contract}/daily-usage")]
async fn list_daily_contract_usage(
    pool: web::Data<ConnectionPool>,
    path: web::Path<String>,
    query_parameter: web::Query<PaginationAndFilterParams>,
) -> impl Responder {
    let contract = path.into_inner();

    let (page, per_page, sort_by, order) = match normalize_pagination(&query_parameter) {
        Ok(result) => result,
        Err(response) => return response,
    };

    let (date_init, date_end) = match normalize_filter(&query_parameter) {
        Ok(result) => result,
        Err(response) => return response,
    };

    let conn = &pool.connection.get().unwrap();
    let usage_data = internals::list_daily_contract_usage_internal(
        conn,
        contract.clone(),
        page,
        per_page,
        sort_by.as_deref(),
        order.as_deref(),
        date_init.clone(),
        date_end.clone(),
    );

    match usage_data {
        Ok(ud) => HttpResponse::Ok().json(PagedResp::new(
            ud,
            internals::count_daily_contract_usage_internal(
                conn,
                contract.clone(),
                date_init.clone(),
                date_end.clone(),
            ),
        )),
        Err(err) => HttpResponse::Ok().json(err),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list_balance_history);
    cfg.service(list_token_price_history);
    cfg.service(list_daily_contract_usage);
}
