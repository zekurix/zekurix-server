use super::Permission;

#[allow(dead_code)]
pub trait RequiredPermission {
    const PERMISSION: Permission;
}

#[allow(dead_code)]
pub struct RolesCreate;

impl RequiredPermission for RolesCreate {
    const PERMISSION: Permission = Permission::RolesCreate;
}

#[allow(dead_code)]
pub struct RolesRead;

impl RequiredPermission for RolesRead {
    const PERMISSION: Permission = Permission::RolesRead;
}

#[allow(dead_code)]
pub struct RolesUpdate;

impl RequiredPermission for RolesUpdate {
    const PERMISSION: Permission = Permission::RolesUpdate;
}

#[allow(dead_code)]
pub struct RolesDelete;

impl RequiredPermission for RolesDelete {
    const PERMISSION: Permission = Permission::RolesDelete;
}

#[allow(dead_code)]
pub struct RolesAssign;

impl RequiredPermission for RolesAssign {
    const PERMISSION: Permission = Permission::RolesAssign;
}

#[allow(dead_code)]
pub struct RolesUnassign;

impl RequiredPermission for RolesUnassign {
    const PERMISSION: Permission = Permission::RolesUnassign;
}

#[allow(dead_code)]
pub struct UsersCreate;

impl RequiredPermission for UsersCreate {
    const PERMISSION: Permission = Permission::UsersCreate;
}

#[allow(dead_code)]
pub struct UsersRead;

impl RequiredPermission for UsersRead {
    const PERMISSION: Permission = Permission::UsersRead;
}

#[allow(dead_code)]
pub struct UsersDelete;

impl RequiredPermission for UsersDelete {
    const PERMISSION: Permission = Permission::UsersDelete;
}
