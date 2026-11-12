use axum::http::StatusCode;
use axum_test::TestResponse;
use serde::Deserialize;

use zekurix_server::user::UserId;

use crate::common::assert_problem;

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

pub fn assert_user_not_found(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::NOT_FOUND,
        "https://api.zekurix.com/problems/user/not-found",
    );
}

pub fn assert_user_already_exists(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::CONFLICT,
        "https://api.zekurix.com/problems/user/already-exists",
    );
}
