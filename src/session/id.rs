use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_generates_sessionid_unique_ids() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();

        assert_ne!(id1, id2);
    }

    #[test]
    fn serde_roundtrip_preserves_sessionid_value() {
        let id = SessionId::new();
        let json = serde_json::to_string(&id).unwrap();
        let back: SessionId = serde_json::from_str(&json).unwrap();

        assert_eq!(id, back);
    }

    #[test]
    fn display_sessionid_produces_valid_uuid_string() {
        let id = SessionId::new();
        let s = id.to_string();

        assert_eq!(s.len(), 36);
        assert!(s.parse::<Uuid>().is_ok());
    }

    #[test]
    fn copy_does_not_change_sessionid_value() {
        let id1 = SessionId::new();
        let id2 = id1;

        assert_eq!(id1, id2);
    }
}
