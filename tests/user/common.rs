use serde::Deserialize;

use zekurix_server::user::UserId;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UserResponse {
    pub id: UserId,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsersResponse {
    pub users: Vec<UserResponse>,
}
