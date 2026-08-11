use uuid::Uuid;

#[derive(Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
}

impl User {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
        }
    }
}
