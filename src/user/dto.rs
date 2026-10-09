use serde::{Deserialize, Serialize};

use crate::identity::dto;

use super::{User, UserId};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateUserRequest {
    pub identity: dto::Identity,
}

#[derive(Serialize)]
pub struct UserResponse {
    id: UserId,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self { id: user.id }
    }
}
