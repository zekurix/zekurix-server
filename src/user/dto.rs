use serde::{Deserialize, Serialize};

use crate::identity::dto;

use super::{User, UserId, Username};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateUserRequest {
    pub identity: dto::Identity,
    pub username: Username,
}

#[derive(Serialize)]
pub struct UserResponse {
    id: UserId,
    username: Username,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
        }
    }
}
