use zekurix_server::identity::{Identity, Issuer, Subject};
use zekurix_server::user::User;

pub(super) fn user_with_identity(issuer: &str, subject: &str) -> (User, Identity) {
    let user = User::new();
    let identity = Identity::new(
        Issuer::new(issuer).unwrap(),
        Subject::new(subject).unwrap(),
        user.id,
    );

    (user, identity)
}
