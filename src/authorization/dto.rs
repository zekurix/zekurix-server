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
pub struct RoleRequest {
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

#[derive(Serialize)]
pub struct RolesResponse {
    roles: Vec<RoleResponse>,
}

impl From<Vec<Role>> for RolesResponse {
    fn from(roles: Vec<Role>) -> Self {
        Self {
            roles: roles.into_iter().map(Into::into).collect(),
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
    fn should_deserialize_role_request() {
        let json = r#"{"name":"admin","permissions":["roles:create","roles:read"]}"#;

        let request: RoleRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.name, RoleName::new("admin").unwrap());
        assert_eq!(
            request.permissions,
            vec![Permission::RolesCreate, Permission::RolesRead]
        );
    }

    #[test]
    fn should_reject_unknown_fields_in_role_request() {
        let json = r#"{"name":"admin","permissions":["roles:create"],"unexpected":true}"#;

        let result: Result<RoleRequest, _> = serde_json::from_str(json);

        assert!(result.is_err());
    }

    #[test]
    fn should_convert_roles_to_roles_response() {
        let first_id = RoleId::new();
        let second_id = RoleId::new();
        let first_name = RoleName::new("admin").unwrap();
        let second_name = RoleName::new("reader").unwrap();
        let first_permissions = vec![Permission::RolesRead];
        let second_permissions = vec![Permission::RolesCreate, Permission::RolesRead];

        let roles = vec![
            Role::from_parts(first_id, first_name.clone(), first_permissions.clone()),
            Role::from_parts(second_id, second_name.clone(), second_permissions.clone()),
        ];

        let response = RolesResponse::from(roles);

        assert_eq!(response.roles.len(), 2);
        assert_eq!(response.roles[0].id, first_id);
        assert_eq!(response.roles[0].name, first_name);
        assert_eq!(response.roles[0].permissions, first_permissions);
        assert_eq!(response.roles[1].id, second_id);
        assert_eq!(response.roles[1].name, second_name);
        assert_eq!(response.roles[1].permissions, second_permissions);
    }

    #[test]
    fn should_convert_empty_roles_to_roles_response() {
        let response = RolesResponse::from(vec![]);

        assert!(response.roles.is_empty());
    }
}
