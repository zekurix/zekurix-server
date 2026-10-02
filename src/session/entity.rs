use time::OffsetDateTime;

use crate::user::UserId;

use super::id::SessionId;

#[derive(Clone, Debug)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    expires_at: OffsetDateTime,
}

impl Session {
    pub fn new(user_id: UserId, expires_at: OffsetDateTime) -> Self {
        Self {
            id: SessionId::new(),
            user_id,
            expires_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at <= OffsetDateTime::now_utc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Duration;

    #[test]
    fn should_create_valid_session() {
        let user_id = UserId::new();
        let expires_at = OffsetDateTime::now_utc() + Duration::days(30);

        let session = Session::new(user_id, expires_at);

        assert_ne!(session.id, SessionId::nil());
        assert_eq!(session.user_id, user_id);
    }

    #[test]
    fn should_generate_different_id_for_each_session() {
        let user_id = UserId::new();
        let expires_at = OffsetDateTime::now_utc() + Duration::days(30);

        let session1 = Session::new(user_id, expires_at);
        let session2 = Session::new(user_id, expires_at);

        assert_ne!(session1.id, session2.id);
    }

    #[test]
    fn should_not_be_expired_when_expiration_is_in_future() {
        let user_id = UserId::new();
        let expires_at = OffsetDateTime::now_utc() + Duration::days(30);

        let session = Session::new(user_id, expires_at);

        assert!(!session.is_expired());
    }

    #[test]
    fn should_be_expired_when_expiration_is_in_past() {
        let user_id = UserId::new();
        let expires_at = OffsetDateTime::now_utc() - Duration::days(1);

        let session = Session::new(user_id, expires_at);

        assert!(session.is_expired());
    }
}
