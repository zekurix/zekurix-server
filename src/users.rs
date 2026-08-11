use std::collections::HashMap;
use std::sync::Mutex;

use uuid::Uuid;

use crate::user::User;

#[derive(Default)]
pub struct Users {
    map: Mutex<HashMap<Uuid, User>>,
}

impl Users {
    pub fn find(&self, id: Uuid) -> Option<User> {
        self.map.lock().unwrap().get(&id).cloned()
    }

    pub fn create(&self, user: User) {
        self.map.lock().unwrap().insert(user.id, user);
    }
}
