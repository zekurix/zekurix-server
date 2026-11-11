use std::collections::HashSet;

use crate::authorization::Permission;
use crate::error::{Error, Result};

use super::{RoleId, RoleName};

#[derive(Debug)]
pub struct Role {
    id: RoleId,
    name: RoleName,
    permissions: Vec<Permission>,
}

impl Role {
    fn validate_unique_permissions(permissions: &[Permission]) -> Result<()> {
        let mut seen = HashSet::with_capacity(permissions.len());

        for &permission in permissions {
            if !seen.insert(permission) {
                return Err(Error::DuplicatePermission(permission));
            }
        }

        Ok(())
    }

    pub fn new(name: RoleName, permissions: Vec<Permission>) -> Self {
        Self {
            id: RoleId::new(),
            name,
            permissions,
        }
    }

    pub fn try_new(name: RoleName, permissions: Vec<Permission>) -> Result<Self> {
        Self::validate_unique_permissions(&permissions)?;
        Ok(Self::new(name, permissions))
    }

    pub fn from_parts(id: RoleId, name: RoleName, permissions: Vec<Permission>) -> Self {
        Self {
            id,
            name,
            permissions,
        }
    }

    pub fn try_from_parts(
        id: RoleId,
        name: RoleName,
        permissions: Vec<Permission>,
    ) -> Result<Self> {
        Self::validate_unique_permissions(&permissions)?;
        Ok(Self::from_parts(id, name, permissions))
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

    pub fn into_parts(self) -> (RoleId, RoleName, Vec<Permission>) {
        (self.id, self.name, self.permissions)
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
    fn try_new_creates_a_role_with_unique_permissions() {
        let name = role_name();
        let permissions = vec![Permission::RolesRead, Permission::RolesUpdate];

        let role = Role::try_new(name.clone(), permissions.clone()).unwrap();

        assert_ne!(role.id(), RoleId::nil());
        assert_eq!(role.name(), &name);
        assert_eq!(role.permissions(), permissions);
    }

    #[test]
    fn try_new_rejects_duplicate_permissions() {
        let result = Role::try_new(
            role_name(),
            vec![Permission::RolesRead, Permission::RolesRead],
        );

        assert!(matches!(
            result,
            Err(Error::DuplicatePermission(Permission::RolesRead))
        ));
    }

    #[test]
    fn try_new_accepts_no_permissions() {
        let role = Role::try_new(role_name(), vec![]).unwrap();

        assert!(role.permissions().is_empty());
    }

    #[test]
    fn from_parts_preserves_the_role_data() {
        let id = RoleId::nil();
        let name = role_name();
        let permissions = vec![Permission::RolesRead, Permission::RolesDelete];

        let role = Role::from_parts(id, name.clone(), permissions.clone());

        assert_eq!(role.id(), id);
        assert_eq!(role.name(), &name);
        assert_eq!(role.permissions(), permissions);
    }

    #[test]
    fn try_from_parts_preserves_role_data_with_unique_permissions() {
        let id = RoleId::nil();
        let name = role_name();
        let permissions = vec![Permission::RolesRead, Permission::RolesDelete];

        let role = Role::try_from_parts(id, name.clone(), permissions.clone()).unwrap();

        assert_eq!(role.id(), id);
        assert_eq!(role.name(), &name);
        assert_eq!(role.permissions(), permissions);
    }

    #[test]
    fn try_from_parts_rejects_duplicate_permissions() {
        let result = Role::try_from_parts(
            RoleId::nil(),
            role_name(),
            vec![Permission::RolesRead, Permission::RolesRead],
        );

        assert!(matches!(
            result,
            Err(Error::DuplicatePermission(Permission::RolesRead))
        ));
    }

    #[test]
    fn has_permission_reports_whether_the_permission_is_present() {
        let role = Role::new(role_name(), vec![Permission::RolesRead]);

        assert!(role.has_permission(Permission::RolesRead));
        assert!(!role.has_permission(Permission::RolesCreate));
    }

    #[test]
    fn into_parts_moves_the_role_data_out() {
        let id = RoleId::nil();
        let name = role_name();
        let permissions = vec![Permission::RolesRead, Permission::RolesDelete];

        let role = Role::from_parts(id, name.clone(), permissions.clone());
        let (actual_id, actual_name, actual_permissions) = role.into_parts();

        assert_eq!(actual_id, id);
        assert_eq!(actual_name, name);
        assert_eq!(actual_permissions, permissions);
    }
}
