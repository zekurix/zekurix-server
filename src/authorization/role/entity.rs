use crate::authorization::Permission;

use super::{RoleId, RoleName};

pub struct Role {
    id: RoleId,
    name: RoleName,
    permissions: Vec<Permission>,
}

impl Role {
    pub fn new(name: RoleName, permissions: Vec<Permission>) -> Self {
        Self {
            id: RoleId::new(),
            name,
            permissions,
        }
    }

    pub fn from_persisted(id: RoleId, name: RoleName, permissions: Vec<Permission>) -> Self {
        Self {
            id,
            name,
            permissions,
        }
    }

    pub fn id(&self) -> RoleId {
        self.id
    }

    pub fn name(&self) -> &RoleName {
        &self.name
    }

    pub fn permissions(&self) -> &[Permission] {
        &self.permissions
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn role_name() -> RoleName {
        RoleName::new("test-role").unwrap()
    }

    #[test]
    fn new_creates_a_role_with_its_name_and_permissions() {
        let name = role_name();
        let permissions = vec![Permission::RolesRead, Permission::RolesUpdate];

        let role = Role::new(name.clone(), permissions.clone());

        assert_ne!(role.id(), RoleId::nil());
        assert_eq!(role.name(), &name);
        assert_eq!(role.permissions(), permissions);
    }

    #[test]
    fn from_persisted_preserves_the_role_data() {
        let id = RoleId::nil();
        let name = role_name();
        let permissions = vec![Permission::RolesRead, Permission::RolesDelete];

        let role = Role::from_persisted(id, name.clone(), permissions.clone());

        assert_eq!(role.id(), id);
        assert_eq!(role.name(), &name);
        assert_eq!(role.permissions(), permissions);
    }

    #[test]
    fn has_permission_reports_whether_the_permission_is_present() {
        let role = Role::new(role_name(), vec![Permission::RolesRead]);

        assert!(role.has_permission(Permission::RolesRead));
        assert!(!role.has_permission(Permission::RolesCreate));
    }
}
