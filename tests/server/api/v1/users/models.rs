use serde::Deserialize;

use crate::server::pagination::PaginationResponse;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct UserResponse {
    pub(super) id: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct UsersResponse {
    pub(super) users: Vec<UserResponse>,
    pub(super) pagination: PaginationResponse,
}
