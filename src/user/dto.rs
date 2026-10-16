use serde::{Deserialize, Serialize};

use crate::identity::{Issuer, Subject};

use super::{User, UserId};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub issuer: Issuer,
    pub subject: Subject,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateUserRequest {
    pub identity: Identity,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_convert_user_to_user_response() {
        let id = UserId::new();
        let user = User { id };

        let response = UserResponse::from(user);

        assert_eq!(response.id, id);
    }
}
