use super::Pagination;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub pagination: Pagination,
    pub total: u64,
}

impl<T> Page<T> {
    pub const fn new(items: Vec<T>, pagination: Pagination, total: u64) -> Self {
        Self {
            items,
            pagination,
            total,
        }
    }
}
