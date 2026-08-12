use std::collections::HashMap;
use std::sync::Mutex;

use uuid::Uuid;

use crate::error::{Error, Result};
use crate::user::User;

#[derive(Default)]
pub struct Users {
    map: Mutex<HashMap<Uuid, User>>,
}

impl Users {
    fn get(&self, id: Uuid) -> Option<User> {
        self.map.lock().unwrap().get(&id).cloned()
    }

    pub fn find(&self, id: Uuid) -> Result<User> {
        self.get(id).ok_or(Error::UserNotFound(id.to_string()))
    }

    pub fn create(&self, user: User) -> Result<()> {
        let mut map = self.map.lock().unwrap();

        if map.values().any(|u| u.username == user.username) {
            return Err(Error::UserAlreadyExists(user.username));
        }

        map.insert(user.id, user);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_return_none_when_user_does_not_exist() {
        let users = Users::default();
        let result = users.get(Uuid::now_v7());

        assert!(result.is_none());
    }

    #[test]
    fn should_find_return_user_not_found_when_user_does_not_exist() {
        let users = Users::default();
        let result = users.find(Uuid::now_v7());

        assert!(matches!(result, Err(Error::UserNotFound(_))));
    }

    #[test]
    fn should_get_created_user() {
        let users = Users::default();
        let user = User::new("Alice".to_string());
        let id = user.id;

        users.create(user).unwrap();

        let found = users.get(id);

        assert!(found.is_some());
        assert_eq!(found.unwrap().id, id);
    }

    #[test]
    fn should_find_created_user() {
        let users = Users::default();
        let user = User::new("Alice".to_string());
        let id = user.id;

        users.create(user).unwrap();

        let found = users.find(id);

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

        assert!(matches!(result, Err(Error::UserAlreadyExists(_))));
    }
}
