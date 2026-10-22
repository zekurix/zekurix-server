use std::sync::Arc;

use tokio::sync::RwLock;

use super::VerificationKeySet;

#[derive(Debug, Default)]
pub struct VerificationKeyStore {
    current: RwLock<Arc<VerificationKeySet>>,
}

impl VerificationKeyStore {
    pub fn new(initial: VerificationKeySet) -> Self {
        Self {
            current: RwLock::new(Arc::new(initial)),
        }
    }

    pub async fn current(&self) -> Arc<VerificationKeySet> {
        let guard = self.current.read().await;
        Arc::clone(&guard)
    }

    pub async fn replace(&self, new: VerificationKeySet) {
        let mut guard = self.current.write().await;
        *guard = Arc::new(new);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::jwk::JwkSet;
    use serde_json::json;

    fn key_set(kid: &str) -> VerificationKeySet {
        let jwks: JwkSet = serde_json::from_value(json!({
            "keys": [{
                "kty": "RSA",
                "kid": kid,
                "alg": "RS256",
                "n": "AQ",
                "e": "AQ"
            }]
        }))
        .unwrap();

        VerificationKeySet::new(jwks)
    }

    #[tokio::test]
    async fn default_store_starts_with_an_empty_key_set() {
        let store = VerificationKeyStore::default();

        assert!(store.current().await.keys.is_empty());
    }

    #[tokio::test]
    async fn new_store_exposes_its_initial_key_set() {
        let store = VerificationKeyStore::new(key_set("initial"));

        assert_eq!(
            store.current().await.keys[0].kid.as_deref(),
            Some("initial")
        );
    }

    #[tokio::test]
    async fn replace_publishes_new_key_set_without_changing_existing_snapshots() {
        let store = VerificationKeyStore::new(key_set("initial"));
        let previous = store.current().await;
        let same_snapshot = store.current().await;

        assert!(Arc::ptr_eq(&previous, &same_snapshot));

        store.replace(key_set("replacement")).await;

        let current = store.current().await;
        assert_eq!(previous.keys[0].kid.as_deref(), Some("initial"));
        assert_eq!(current.keys[0].kid.as_deref(), Some("replacement"));
        assert!(!Arc::ptr_eq(&previous, &current));
    }

    #[tokio::test]
    async fn concurrent_reads_are_safe() {
        let store = Arc::new(VerificationKeyStore::new(key_set("initial")));

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let store = Arc::clone(&store);
                tokio::spawn(async move {
                    let _ = store.current().await;
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn concurrent_read_write_is_safe() {
        let store = Arc::new(VerificationKeyStore::new(key_set("initial")));
        let barrier = Arc::new(tokio::sync::Barrier::new(12));

        let mut readers = Vec::new();

        for _ in 0..10 {
            let store = Arc::clone(&store);
            let barrier = Arc::clone(&barrier);

            readers.push(tokio::spawn(async move {
                barrier.wait().await;
                for _ in 0..100 {
                    let snapshot = store.current().await;
                    assert_eq!(snapshot.keys.len(), 1);
                    assert!(matches!(
                        snapshot.keys[0].kid.as_deref(),
                        Some("initial" | "updated")
                    ));
                    tokio::task::yield_now().await;
                }
            }));
        }

        let writer_store = Arc::clone(&store);
        let writer_barrier = Arc::clone(&barrier);
        let writer = tokio::spawn(async move {
            writer_barrier.wait().await;
            for _ in 0..100 {
                writer_store.replace(key_set("updated")).await;
                tokio::task::yield_now().await;
            }
        });

        barrier.wait().await;

        writer.await.unwrap();
        for reader in readers {
            reader.await.unwrap();
        }

        assert_eq!(
            store.current().await.keys[0].kid.as_deref(),
            Some("updated")
        );
    }
}
