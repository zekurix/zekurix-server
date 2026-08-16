use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Clone)]
pub struct User {
    pub id: UserId,
    pub username: String,
}

impl User {
    pub fn new(username: String) -> Self {
        Self {
            id: UserId::new(),
            username,
        }
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

    #[test]
    fn should_create_user_with_given_name() {
        let user = User::new("Alice".to_string());

        assert_eq!(user.username, "Alice");
    }

    #[test]
    fn should_generate_non_nil_uuid() {
        let user = User::new("Alice".to_string());

        assert_ne!(user.id, UserId::nil());
    }

    #[test]
    fn should_generate_different_uuid_for_each_user() {
        let user1 = User::new("Alice".to_string());
        let user2 = User::new("Alice".to_string());

        assert_ne!(user1.id, user2.id);
    }
}
