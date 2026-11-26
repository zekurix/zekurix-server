use super::PaginationQuery;

pub(super) const DEFAULT_PAGE_LIMIT: u32 = 50;
pub(super) const DEFAULT_PAGE_OFFSET: u32 = 0;
pub(super) const MAX_PAGE_LIMIT: u32 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pagination {
    pub limit: u32,
    pub offset: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: DEFAULT_PAGE_LIMIT,
            offset: DEFAULT_PAGE_OFFSET,
        }
    }
}

impl From<PaginationQuery> for Pagination {
    fn from(query: PaginationQuery) -> Self {
        Self {
            limit: query.limit,
            offset: query.offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_expected_values() {
        let pagination = Pagination::default();

        assert_eq!(pagination.limit, DEFAULT_PAGE_LIMIT);
        assert_eq!(pagination.offset, DEFAULT_PAGE_OFFSET);
    }

    #[test]
    fn from_query_preserves_limit_and_offset() {
        let query = PaginationQuery {
            limit: 25,
            offset: 100,
        };

        let pagination = Pagination::from(query);

        assert_eq!(pagination.limit, 25);
        assert_eq!(pagination.offset, 100);
    }
}
