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
pub struct UsersResponse {
    users: Vec<UserResponse>,
    pagination: PaginationResponse,
}

impl From<Page<User>> for UsersResponse {
    fn from(page: Page<User>) -> Self {
        Self {
            users: page.items.into_iter().map(Into::into).collect(),
            pagination: PaginationResponse::new(page.pagination, page.total),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::router::Pagination;

    use super::*;

    #[test]
    fn should_convert_user_to_user_response() {
        let id = UserId::new();
        let user = User { id };

        let response = UserResponse::from(user);

        assert_eq!(response.id, id);
    }

    #[test]
    fn should_convert_page_to_users_response() {
        let users = vec![User::new(), User::new()];
        let expected_ids = users.iter().map(|user| user.id).collect::<Vec<_>>();
        let pagination = Pagination {
            limit: 25,
            offset: 50,
        };

        let response = UsersResponse::from(Page::new(users, pagination, 100));

        assert_eq!(
            response
                .users
                .iter()
                .map(|user| user.id)
                .collect::<Vec<_>>(),
            expected_ids
        );
        assert_eq!(response.pagination.limit, 25);
        assert_eq!(response.pagination.offset, 50);
        assert_eq!(response.pagination.total, 100);
    }
}
