use zekurix_server::{router::Pagination, user::UserRepository};

use crate::repositories::postgres::fixtures::user_with_identity;

use super::harness::TestHarness;

#[tokio::test]
async fn should_list_users_ordered_by_id() {
    // Persist multiple users and verify that list returns them ordered by user ID.
    let harness = TestHarness::new().await;

    let (first_user, first_identity) = user_with_identity("test-issuer", "subject-1");
    let (second_user, second_identity) = user_with_identity("test-issuer", "subject-2");
    let (third_user, third_identity) = user_with_identity("test-issuer", "subject-3");

    harness
        .repository
        .create(third_user.clone(), third_identity)
        .await
        .unwrap();

    harness
        .repository
        .create(first_user.clone(), first_identity)
        .await
        .unwrap();

    harness
        .repository
        .create(second_user.clone(), second_identity)
        .await
        .unwrap();

    let page = harness
        .repository
        .list(Pagination {
            limit: 100,
            offset: 0,
        })
        .await
        .unwrap();

    let mut expected = vec![first_user, second_user, third_user];
    expected.sort_by_key(|user| user.id);

    assert_eq!(page.items, expected);
    assert_eq!(page.total, 3);
}

#[tokio::test]
async fn should_apply_limit_and_offset_when_listing_users() {
    // Persist multiple users and verify that list applies both the requested
    // limit and offset to the ordered result set.
    let harness = TestHarness::new().await;

    let mut users = Vec::new();

    for index in 0..5 {
        let (user, identity) = user_with_identity("test-issuer", &format!("subject-{index}"));

        harness
            .repository
            .create(user.clone(), identity)
            .await
            .unwrap();

        users.push(user);
    }

    users.sort_by_key(|user| user.id);

    let pagination = Pagination {
        limit: 2,
        offset: 1,
    };

    let page = harness.repository.list(pagination).await.unwrap();

    assert_eq!(page.items, users[1..3]);
    assert_eq!(page.pagination, pagination);
    assert_eq!(page.total, users.len() as u64);
}
