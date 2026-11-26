use serde::Deserialize;

use crate::server::pagination::PaginationResponse;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoleResponse {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) permissions: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct RolesResponse {
    pub(super) roles: Vec<RoleResponse>,
    pub(super) pagination: PaginationResponse,
}
