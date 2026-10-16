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
    fn new_generate_unique_user_ids() {
        let id1 = UserId::new();
        let id2 = UserId::new();

        assert_ne!(id1, id2);
    }

    #[test]
    fn should_preserve_user_id_through_serde_roundtrip() {
        let id = UserId::new();
        let json = serde_json::to_string(&id).unwrap();
        let back: UserId = serde_json::from_str(&json).unwrap();

        assert_eq!(id, back);
    }

    #[test]
    fn should_display_user_id_as_uuid() {
        let id = UserId::new();

        assert!(id.to_string().parse::<Uuid>().is_ok());
    }
}
