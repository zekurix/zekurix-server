use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(transparent)]
pub struct RoleId(Uuid);

impl RoleId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}

impl Default for RoleId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RoleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_unique_role_ids() {
        let id1 = RoleId::new();
        let id2 = RoleId::new();

        assert_ne!(id1, id2);
    }

    #[test]
    fn should_preserve_role_id_through_serde_roundtrip() {
        let id = RoleId::new();
        let json = serde_json::to_string(&id).unwrap();
        let back: RoleId = serde_json::from_str(&json).unwrap();

        assert_eq!(id, back);
    }

    #[test]
    fn should_display_role_id_as_uuid() {
        let id = RoleId::new();

        assert!(id.to_string().parse::<Uuid>().is_ok());
    }
}
