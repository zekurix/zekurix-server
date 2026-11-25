use serde::Deserialize;

use super::RoleName;

#[derive(Debug, Deserialize)]
pub struct RoleQuery {
    pub name: Option<RoleName>,
}
