#[derive(thiserror::Error, Debug)]
pub enum PaginationError {
    #[error("Invalid page or page_size: page must be at least 0 and page_size must be at least 1")]
    InvalidPageOrPageSize,
}

pub fn get_pagination_params(
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<(i64, i64), PaginationError> {
    let page = page.unwrap_or(0);
    let page_size = page_size.unwrap_or(10);
    if page_size < 1 || page < 0 {
        Err(PaginationError::InvalidPageOrPageSize)?;
    }
    let offset = page * page_size;

    Ok((page_size, offset))
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    #[test]
    fn test_get_pagination_params() {
        assert_eq!(get_pagination_params(None, None).unwrap(), (10, 0));
        assert_eq!(get_pagination_params(Some(0), Some(10)).unwrap(), (10, 0));
        assert_eq!(get_pagination_params(Some(1), Some(10)).unwrap(), (10, 10));
        assert_eq!(get_pagination_params(Some(1), Some(5)).unwrap(), (5, 5));
        assert_eq!(get_pagination_params(Some(2), Some(5)).unwrap(), (5, 10));
        assert_eq!(get_pagination_params(Some(13), Some(7)).unwrap(), (7, 91));
        assert_matches!(
            get_pagination_params(Some(0), Some(0)),
            Err(PaginationError::InvalidPageOrPageSize)
        );
        assert_matches!(
            get_pagination_params(Some(-1), Some(10)),
            Err(PaginationError::InvalidPageOrPageSize)
        );
    }
}
