use uuid::Uuid;

#[derive(Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
}

impl User {
    pub fn new(username: String) -> Self {
        Self {
            id: Uuid::now_v7(),
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

        assert_ne!(user.id, Uuid::nil());
    }

    #[test]
    fn should_generate_different_uuid_for_each_user() {
        let user1 = User::new("Alice".to_string());
        let user2 = User::new("Alice".to_string());

        assert_ne!(user1.id, user2.id);
    }
}
