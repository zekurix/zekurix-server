use super::id::UserId;
use super::username::Username;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct User {
    pub id: UserId,
    pub username: Username,
}

impl User {
    pub fn new(username: Username) -> Self {
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
    fn should_create_user_with_given_name() {
        let user = User::new(Username::new("Alice").unwrap());

        assert_eq!(user.username, Username::new("Alice").unwrap());
    }

    #[test]
    fn should_generate_non_nil_uuid() {
        let user = User::new(Username::new("Alice").unwrap());

        assert_ne!(user.id, UserId::nil());
    }

    #[test]
    fn should_generate_different_uuid_for_each_user() {
        let user1 = User::new(Username::new("Alice").unwrap());
        let user2 = User::new(Username::new("Alice").unwrap());

        assert_ne!(user1.id, user2.id);
    }
}
