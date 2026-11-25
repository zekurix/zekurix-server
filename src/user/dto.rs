use serde::{Deserialize, Serialize};

use crate::identity::{Issuer, Subject};
use crate::router::{Page, PaginationResponse};

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
pub struct UsersPageResponse {
    users: Vec<UserResponse>,
    pagination: PaginationResponse,
}

impl From<Page<User>> for UsersPageResponse {
    fn from(page: Page<User>) -> Self {
        Self {
            users: page.items.into_iter().map(Into::into).collect(),
            pagination: PaginationResponse {
                limit: page.query.limit,
                offset: page.query.offset,
                total: page.total,
            },
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
}
