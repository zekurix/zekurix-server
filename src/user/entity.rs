use super::id::UserId;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct User {
    pub id: UserId,
}

impl User {
    pub fn new() -> Self {
        Self { id: UserId::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_non_nil_uuid() {
        let user = User::new();

        assert_ne!(user.id, UserId::nil());
    }

    #[test]
    fn should_generate_different_uuid_for_each_user() {
        let user1 = User::new();
        let user2 = User::new();

        assert_ne!(user1.id, user2.id);
    }
}
