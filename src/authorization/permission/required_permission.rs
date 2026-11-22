use super::Permission;

pub trait RequiredPermission {
    const PERMISSION: Permission;
}

pub struct RolesCreate;

impl RequiredPermission for RolesCreate {
    const PERMISSION: Permission = Permission::RolesCreate;
}

pub struct RolesRead;

impl RequiredPermission for RolesRead {
    const PERMISSION: Permission = Permission::RolesRead;
}

pub struct RolesUpdate;

impl RequiredPermission for RolesUpdate {
    const PERMISSION: Permission = Permission::RolesUpdate;
}

pub struct RolesDelete;

impl RequiredPermission for RolesDelete {
    const PERMISSION: Permission = Permission::RolesDelete;
}

pub struct RolesAssign;

impl RequiredPermission for RolesAssign {
    const PERMISSION: Permission = Permission::RolesAssign;
}

pub struct RolesUnassign;

impl RequiredPermission for RolesUnassign {
    const PERMISSION: Permission = Permission::RolesUnassign;
}

pub struct UsersCreate;

impl RequiredPermission for UsersCreate {
    const PERMISSION: Permission = Permission::UsersCreate;
}

pub struct UsersRead;

impl RequiredPermission for UsersRead {
    const PERMISSION: Permission = Permission::UsersRead;
}

#[allow(dead_code)]
pub struct UsersDelete;

impl RequiredPermission for UsersDelete {
    const PERMISSION: Permission = Permission::UsersDelete;
}
