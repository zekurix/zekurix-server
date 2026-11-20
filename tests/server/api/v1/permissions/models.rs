use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PermissionsResponse {
    pub(crate) permissions: Vec<String>,
}
