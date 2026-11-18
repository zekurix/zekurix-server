use zekurix_server::authorization::AuthorizationBootstrapRepository;

use super::common::TestRepository;

const BOOTSTRAP_NAME: &str = "test-authorization";

#[tokio::test]
async fn should_report_bootstrap_as_not_completed() {
    // A new bootstrap must initially be reported as incomplete.
    let test = TestRepository::new().await;

    let mut tx = test.repository.begin().await.unwrap();
    let completed = tx.is_completed(BOOTSTRAP_NAME).await.unwrap();

    assert!(!completed);
}

#[tokio::test]
async fn should_report_bootstrap_as_completed() {
    // A bootstrap marked as completed and committed must be reported as completed
    // by a subsequent transaction.
    let test = TestRepository::new().await;

    let mut tx = test.repository.begin().await.unwrap();
    tx.mark_completed(BOOTSTRAP_NAME).await.unwrap();
    tx.commit().await.unwrap();

    let mut tx = test.repository.begin().await.unwrap();
    let completed = tx.is_completed(BOOTSTRAP_NAME).await.unwrap();

    assert!(completed);
}

#[tokio::test]
async fn should_rollback_completion_when_transaction_is_dropped() {
    // Marking a bootstrap as completed without committing the transaction must
    // roll back the completion marker when the transaction is dropped.
    let test = TestRepository::new().await;

    let mut tx = test.repository.begin().await.unwrap();
    tx.mark_completed(BOOTSTRAP_NAME).await.unwrap();
    drop(tx);

    let mut tx = test.repository.begin().await.unwrap();
    let completed = tx.is_completed(BOOTSTRAP_NAME).await.unwrap();

    assert!(!completed);
}
