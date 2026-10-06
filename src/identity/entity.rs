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
