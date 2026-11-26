use serde::Serialize;

use super::Pagination;

#[derive(Serialize)]
pub struct PaginationResponse {
    pub limit: u32,
    pub offset: u32,
    pub total: u64,
}

impl PaginationResponse {
    pub const fn new(pagination: Pagination, total: u64) -> Self {
        Self {
            limit: pagination.limit,
            offset: pagination.offset,
            total,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_preserves_pagination_and_total() {
        let pagination = Pagination {
            limit: 25,
            offset: 100,
        };

        let response = PaginationResponse::new(pagination, 200);

        assert_eq!(response.limit, 25);
        assert_eq!(response.offset, 100);
        assert_eq!(response.total, 200);
    }
}
