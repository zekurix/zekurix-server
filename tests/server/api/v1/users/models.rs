use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct UserResponse {
    pub(super) id: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct PaginationResponse {
    pub(super) limit: u32,
    pub(super) offset: u32,
    pub(super) total: u64,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct UsersResponse {
    pub(super) users: Vec<UserResponse>,
    pub(super) pagination: PaginationResponse,
}
