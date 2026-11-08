use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    #[serde(rename = "permissions:read")]
    PermissionsRead,
    #[serde(rename = "roles:create")]
    RolesCreate,
    #[serde(rename = "roles:read")]
    RolesRead,
    #[serde(rename = "roles:update")]
    RolesUpdate,
    #[serde(rename = "roles:delete")]
    RolesDelete,
    #[serde(rename = "roles:assign")]
    RolesAssign,
    #[serde(rename = "roles:unassign")]
    RolesUnassign,
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::PermissionsRead => "permissions:read",
            Self::RolesCreate => "roles:create",
            Self::RolesRead => "roles:read",
            Self::RolesUpdate => "roles:update",
            Self::RolesDelete => "roles:delete",
            Self::RolesAssign => "roles:assign",
            Self::RolesUnassign => "roles:unassign",
        };

        write!(f, "{value}")
    }
}

impl FromStr for Permission {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "permissions:read" => Ok(Self::PermissionsRead),
            "roles:create" => Ok(Self::RolesCreate),
            "roles:read" => Ok(Self::RolesRead),
            "roles:update" => Ok(Self::RolesUpdate),
            "roles:delete" => Ok(Self::RolesDelete),
            "roles:assign" => Ok(Self::RolesAssign),
            "roles:unassign" => Ok(Self::RolesUnassign),
            _ => Err(Error::InvalidPermission(value.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Permission;
    use std::str::FromStr;

    const PERMISSIONS: [(Permission, &str); 7] = [
        (Permission::PermissionsRead, "permissions:read"),
        (Permission::RolesCreate, "roles:create"),
        (Permission::RolesRead, "roles:read"),
        (Permission::RolesUpdate, "roles:update"),
        (Permission::RolesDelete, "roles:delete"),
        (Permission::RolesAssign, "roles:assign"),
        (Permission::RolesUnassign, "roles:unassign"),
    ];

    #[test]
    fn displays_each_permission() {
        for (permission, expected) in PERMISSIONS {
            assert_eq!(permission.to_string(), expected);
        }
    }

    #[test]
    fn parses_each_permission() {
        for (expected, value) in PERMISSIONS {
            assert_eq!(Permission::from_str(value).unwrap(), expected);
        }
    }

    #[test]
    fn rejects_unknown_permission() {
        let value = "permissions:invalid";

        assert!(matches!(
            Permission::from_str(value),
            Err(crate::error::Error::InvalidPermission(invalid)) if invalid == value
        ));
    }

    #[test]
    fn serializes_and_deserializes_each_permission() {
        for (permission, value) in PERMISSIONS {
            let json = serde_json::to_string(&permission).unwrap();
            assert_eq!(json, format!("\"{value}\""));
            assert_eq!(
                serde_json::from_str::<Permission>(&json).unwrap(),
                permission
            );
        }
    }
}
