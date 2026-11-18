use std::time::Duration;

use tokio::time::{sleep, timeout};
use zekurix_server::authorization::{
    AuthorizationBootstrapRepository, PostgresAuthorizationBootstrapRepository,
};

use super::common::TestRepository;

const BOOTSTRAP_NAME: &str = "test-authorization";
const BLOCKING_TIMEOUT: Duration = Duration::from_millis(100);
const COMPLETION_TIMEOUT: Duration = Duration::from_secs(2);

#[tokio::test]
async fn should_serialize_concurrent_bootstrap_transactions() {
    // A second bootstrap transaction must wait while another transaction holds
    // the authorization bootstrap advisory lock.
    let test = TestRepository::new().await;

    let first_tx = test.repository.begin().await.unwrap();

    let second_repository = PostgresAuthorizationBootstrapRepository::new(test.pool.clone());

    let second = timeout(BLOCKING_TIMEOUT, second_repository.begin()).await;

    assert!(second.is_err());

    drop(first_tx);

    let second = timeout(COMPLETION_TIMEOUT, second_repository.begin())
        .await
        .unwrap()
        .unwrap();

    drop(second);
}

#[tokio::test]
async fn should_observe_completed_bootstrap_after_waiting_for_concurrent_transaction() {
    // After waiting for the winning bootstrap transaction, the second transaction
    // must observe that the bootstrap has already been completed.
    let test = TestRepository::new().await;

    let mut first_tx = test.repository.begin().await.unwrap();

    first_tx.mark_completed(BOOTSTRAP_NAME).await.unwrap();

    let second_repository = PostgresAuthorizationBootstrapRepository::new(test.pool.clone());

    let second_handle = tokio::spawn(async move {
        let mut tx = second_repository.begin().await.unwrap();
        let completed = tx.is_completed(BOOTSTRAP_NAME).await.unwrap();

        (tx, completed)
    });

    // Give the second transaction time to reach the advisory lock. It must
    // remain blocked until the first transaction commits.
    sleep(BLOCKING_TIMEOUT).await;

    assert!(!second_handle.is_finished());

    first_tx.commit().await.unwrap();

    let (second_tx, completed) = timeout(COMPLETION_TIMEOUT, second_handle)
        .await
        .unwrap()
        .unwrap();

    assert!(completed);

    drop(second_tx);
}

#[tokio::test]
async fn should_release_bootstrap_lock_when_transaction_is_dropped() {
    // Dropping the transaction without committing must release the advisory lock
    // so another bootstrap transaction can proceed.
    let test = TestRepository::new().await;

    let first_tx = test.repository.begin().await.unwrap();

    let second_repository = PostgresAuthorizationBootstrapRepository::new(test.pool.clone());

    let second_handle = tokio::spawn(async move { second_repository.begin().await.unwrap() });

    // Give the second transaction time to reach the advisory lock and verify
    // that it remains blocked while the first transaction is alive.
    sleep(BLOCKING_TIMEOUT).await;

    assert!(!second_handle.is_finished());

    drop(first_tx);

    let second_tx = timeout(COMPLETION_TIMEOUT, second_handle)
        .await
        .unwrap()
        .unwrap();

    drop(second_tx);
}
