use serde::Serialize;

use super::page::Page;

#[derive(Serialize)]
pub struct PaginationResponse {
    pub limit: u32,
    pub offset: u32,
    pub total: u64,
}

#[derive(Serialize)]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    pub pagination: PaginationResponse,
}

impl<T> From<Page<T>> for PageResponse<T> {
    fn from(page: Page<T>) -> Self {
        Self {
            items: page.items,
            pagination: PaginationResponse {
                limit: page.query.limit,
                offset: page.query.offset,
                total: page.total,
            },
        }
    }
}
