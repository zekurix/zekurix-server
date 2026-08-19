use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_generates_userid_unique_ids() {
        let id1 = UserId::new();
        let id2 = UserId::new();

        assert_ne!(id1, id2);
    }

    #[test]
    fn serde_roundtrip_preserves_userid_value() {
        let id = UserId::new();
        let json = serde_json::to_string(&id).unwrap();
        let back: UserId = serde_json::from_str(&json).unwrap();

        assert_eq!(id, back);
    }

    #[test]
    fn display_userid_produces_valid_uuid_string() {
        let id = UserId::new();
        let s = id.to_string();

        assert_eq!(s.len(), 36);
        assert!(s.parse::<Uuid>().is_ok());
    }

    #[test]
    fn copy_does_not_change_userid_value() {
        let id1 = UserId::new();
        let id2 = id1;

        assert_eq!(id1, id2);
    }
}
