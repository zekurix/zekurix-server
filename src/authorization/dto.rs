use serde::Serialize;

use super::Permission;

#[derive(Serialize)]
pub struct PermissionsResponse {
    permissions: Vec<Permission>,
}

impl From<Vec<Permission>> for PermissionsResponse {
    fn from(permissions: Vec<Permission>) -> Self {
        Self { permissions }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_convert_permissions_to_permissions_response() {
        let permissions = vec![Permission::RolesCreate, Permission::RolesRead];

        let response = PermissionsResponse::from(permissions.clone());

        assert_eq!(response.permissions, permissions);
    }
}
