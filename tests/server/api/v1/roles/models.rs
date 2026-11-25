use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoleResponse {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) permissions: Vec<String>,
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
pub(super) struct RolesResponse {
    pub(super) roles: Vec<RoleResponse>,
    pub(super) pagination: PaginationResponse,
}
