use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PaginationResponse {
    pub(crate) limit: u32,
    pub(crate) offset: u32,
    pub(crate) total: u64,
}
