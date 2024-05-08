#[derive(thiserror::Error, Debug)]
pub enum PaginationError {
    #[error("Invalid page or page_size: page must be at least 0 and page_size must be at least 1")]
    InvalidPageOrPageSize,
}

pub fn get_pagination_params(
    page: Option<i64>,
    page_size: Option<i64>,
) -> anyhow::Result<(i64, i64), PaginationError> {
    let page = page.unwrap_or(0);
    let page_size = page_size.unwrap_or(10);
    if page_size < 1 || page < 0 {
        Err(PaginationError::InvalidPageOrPageSize)?;
    }
    let offset = page * page_size;

    Ok((page_size, offset))
}
