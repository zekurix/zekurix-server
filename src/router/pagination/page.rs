use super::query::PaginationQuery;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub query: PaginationQuery,
    pub total: u64,
}

impl<T> Page<T> {
    pub const fn new(items: Vec<T>, query: PaginationQuery, total: u64) -> Self {
        Self {
            items,
            query,
            total,
        }
    }
}
