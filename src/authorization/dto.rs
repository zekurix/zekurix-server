use serde::{Deserialize, Serialize};

use super::{Permission, Role, RoleId, RoleName};

#[derive(Serialize)]
pub struct PermissionsResponse {
    permissions: Vec<Permission>,
}

impl From<Vec<Permission>> for PermissionsResponse {
    fn from(permissions: Vec<Permission>) -> Self {
        Self { permissions }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateRoleRequest {
    pub name: RoleName,
    pub permissions: Vec<Permission>,
}

#[derive(Serialize)]
pub struct RoleResponse {
    id: RoleId,
    name: RoleName,
    permissions: Vec<Permission>,
}

impl From<Role> for RoleResponse {
    fn from(role: Role) -> Self {
        let (id, name, permissions) = role.into_parts();

        Self {
            id,
            name,
            permissions,
        }
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

    #[test]
    fn should_convert_role_to_role_response() {
        let name = RoleName::new("admin").unwrap();
        let permissions = vec![Permission::RolesCreate, Permission::RolesRead];
        let role = Role::new(name.clone(), permissions.clone());
        let id = role.id();

        let response = RoleResponse::from(role);

        assert_eq!(response.id, id);
        assert_eq!(response.name, name);
        assert_eq!(response.permissions, permissions);
    }

    #[test]
    fn should_deserialize_create_role_request() {
        let json = r#"{"name":"admin","permissions":["roles:create","roles:read"]}"#;

        let request: CreateRoleRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.name, RoleName::new("admin").unwrap());
        assert_eq!(
            request.permissions,
            vec![Permission::RolesCreate, Permission::RolesRead]
        );
    }

    #[test]
    fn should_reject_unknown_fields_in_create_role_request() {
        let json = r#"{"name":"admin","permissions":["roles:create"],"unexpected":true}"#;

        let result: Result<CreateRoleRequest, _> = serde_json::from_str(json);

        assert!(result.is_err());
    }
}
