use serde::Deserialize;

use zekurix_server::user::UserId;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserResponse {
    pub id: UserId,
    pub username: String,
}
