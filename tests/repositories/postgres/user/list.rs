use zekurix_server::{router::Pagination, user::UserRepository};

use crate::repositories::postgres::common::user_with_identity;

use super::common::TestRepository;

#[tokio::test]
async fn should_list_users_ordered_by_id() {
    // Persist multiple users and verify that list returns them ordered by user ID.
    let test = TestRepository::new().await;

    let (first_user, first_identity) = user_with_identity("test-issuer", "subject-1");
    let (second_user, second_identity) = user_with_identity("test-issuer", "subject-2");
    let (third_user, third_identity) = user_with_identity("test-issuer", "subject-3");

    test.repository
        .create(third_user.clone(), third_identity)
        .await
        .unwrap();

    test.repository
        .create(first_user.clone(), first_identity)
        .await
        .unwrap();

    test.repository
        .create(second_user.clone(), second_identity)
        .await
        .unwrap();

    let users = test
        .repository
        .list(Pagination {
            limit: 100,
            offset: 0,
        })
        .await
        .unwrap();

    let mut expected = vec![first_user, second_user, third_user];
    expected.sort_by_key(|user| user.id);

    assert_eq!(users, expected);
}

#[tokio::test]
async fn should_apply_limit_and_offset_when_listing_users() {
    // Persist multiple users and verify that list applies both the requested
    // limit and offset to the ordered result set.
    let test = TestRepository::new().await;

    let mut users = Vec::new();

    for index in 0..5 {
        let (user, identity) = user_with_identity("test-issuer", &format!("subject-{index}"));

        test.repository
            .create(user.clone(), identity)
            .await
            .unwrap();

        users.push(user);
    }

    users.sort_by_key(|user| user.id);

    let listed = test
        .repository
        .list(Pagination {
            limit: 2,
            offset: 1,
        })
        .await
        .unwrap();

    assert_eq!(listed, users[1..3]);
}
