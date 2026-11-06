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

#[derive(Serialize)]
pub struct UsersResponse {
    users: Vec<UserResponse>,
}

impl From<Vec<User>> for UsersResponse {
    fn from(users: Vec<User>) -> Self {
        Self {
            users: users.into_iter().map(Into::into).collect(),
        }
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

    #[test]
    fn should_convert_users_to_users_response() {
        let first_id = UserId::new();
        let second_id = UserId::new();

        let users = vec![User { id: first_id }, User { id: second_id }];

        let response = UsersResponse::from(users);

        assert_eq!(response.users.len(), 2);
        assert_eq!(response.users[0].id, first_id);
        assert_eq!(response.users[1].id, second_id);
    }

    #[test]
    fn should_convert_empty_users_to_users_response() {
        let response = UsersResponse::from(vec![]);

        assert!(response.users.is_empty());
    }
}
