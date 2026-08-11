use std::collections::HashMap;
use std::sync::Mutex;

use uuid::Uuid;

use crate::user::User;

#[derive(Debug, PartialEq)]
pub enum CreateUserError {
    UsernameAlreadyExists,
}

#[derive(Default)]
pub struct Users {
    map: Mutex<HashMap<Uuid, User>>,
}

impl Users {
    pub fn find(&self, id: Uuid) -> Option<User> {
        self.map.lock().unwrap().get(&id).cloned()
    }

    pub fn create(&self, user: User) -> Result<(), CreateUserError> {
        let mut map = self.map.lock().unwrap();

        if map.values().any(|u| u.username == user.username) {
            return Err(CreateUserError::UsernameAlreadyExists);
        }

        map.insert(user.id, user);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_none_when_user_does_not_exist() {
        let users = Users::default();
        let result = users.find(Uuid::now_v7());

        assert!(result.is_none());
    }

    #[test]
    fn should_find_created_user() {
        let users = Users::default();
        let user = User::new("Alice".to_string());
        let id = user.id;

        users.create(user).unwrap();

        let found = users.find(id);

        assert!(found.is_some());
        assert_eq!(found.unwrap().id, id);
    }

    #[test]
    fn should_find_multiple_created_users() {
        let users = Users::default();

        let alice = User::new("Alice".to_string());
        let bob = User::new("Bob".to_string());
        let charlie = User::new("Charlie".to_string());

        let alice_id = alice.id;
        let bob_id = bob.id;
        let charlie_id = charlie.id;

        users.create(alice).unwrap();
        users.create(bob).unwrap();
        users.create(charlie).unwrap();

        let alice = users.find(alice_id).unwrap();
        let bob = users.find(bob_id).unwrap();
        let charlie = users.find(charlie_id).unwrap();

        assert_eq!(alice.username, "Alice");
        assert_eq!(bob.username, "Bob");
        assert_eq!(charlie.username, "Charlie");
    }

    #[test]
    fn should_return_username_already_exists_error() {
        let users = Users::default();

        let alice1 = User::new("Alice".to_string());
        let alice2 = User::new("Alice".to_string());

        assert!(users.create(alice1).is_ok());

        let result = users.create(alice2);

        assert_eq!(result, Err(CreateUserError::UsernameAlreadyExists));
    }
}
