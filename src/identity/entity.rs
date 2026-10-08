use std::fmt;

use crate::user::UserId;

use super::issuer::Issuer;
use super::subject::Subject;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Identity {
    pub issuer: Issuer,
    pub subject: Subject,
    pub user_id: UserId,
}

impl Identity {
    pub fn new(issuer: Issuer, subject: Subject, user_id: UserId) -> Self {
        Self {
            issuer,
            subject,
            user_id,
        }
    }
}

impl fmt::Display for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.subject, self.issuer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_display_identity() {
        let identity = Identity::new(
            Issuer::new("https://auth.example.com").unwrap(),
            Subject::new("Alice").unwrap(),
            UserId::new(),
        );

        assert_eq!(identity.to_string(), "Alice@https://auth.example.com");
    }
}
