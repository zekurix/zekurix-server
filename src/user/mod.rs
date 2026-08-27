mod id;
mod routes;

pub mod handlers;
pub mod postgres_repository;
pub mod repository;
pub mod username;

pub use id::UserId;
pub use routes::router;
pub use username::Username;

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
