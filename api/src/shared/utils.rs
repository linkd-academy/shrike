use crate::error::Error;
use crate::shared::models::{PaginationParams, PAGE_DEFAULT, PER_PAGE_DEFAULT, PER_PAGE_LIMIT};
use actix_web::HttpResponse;

pub fn normalize_pagination(
    query_parameter: &PaginationParams,
) -> Result<(u32, u32, Option<String>, Option<String>), HttpResponse> {
    let page = query_parameter.page.unwrap_or(PAGE_DEFAULT);
    let mut per_page = query_parameter.per_page.unwrap_or(PER_PAGE_DEFAULT);

    if per_page > PER_PAGE_LIMIT {
        per_page = PER_PAGE_LIMIT; // Limit per page
    }

    if per_page == 0 {
        return Err(HttpResponse::BadRequest().json(Error {
            error: "Per_page must be greater than zero.".to_string(),
        }));
    }

    let sort_by = query_parameter.sort_by.clone().filter(|s| !s.is_empty());
    let order = query_parameter
        .order
        .clone()
        .filter(|s| s == "asc" || s == "desc");

    Ok((page, per_page, sort_by, order))
}