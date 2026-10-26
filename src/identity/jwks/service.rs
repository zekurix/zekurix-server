use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use jsonwebtoken::jwk::JwkSet;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

use crate::error::{Error, Result};
use crate::health::{HealthCheck, HealthStatus};
use crate::settings::authentication_provider::Settings;

use super::{JwksProvider, VerificationKeySet, VerificationKeyStore, create_jwks_provider};

pub struct JwksService {
    provider: Arc<dyn JwksProvider>,
    key_store: Arc<VerificationKeyStore>,
    last_loaded_at: RwLock<Option<Instant>>,
    status: RwLock<HealthStatus>,
    startup_retry_timeout: Duration,
    startup_retry_interval: Duration,
    refresh_interval: Duration,
    refresh_retry_interval: Duration,
    stale_keys_ttl: Duration,
}

impl JwksService {
    fn new(settings: &Settings, key_store: Arc<VerificationKeyStore>) -> Self {
        Self::with_provider(settings, key_store, create_jwks_provider(settings))
    }

    fn with_provider(
        settings: &Settings,
        key_store: Arc<VerificationKeyStore>,
        provider: Arc<dyn JwksProvider>,
    ) -> Self {
        Self {
            provider,
            key_store,
            last_loaded_at: RwLock::new(None),
            status: RwLock::new(HealthStatus::Unhealthy),
            startup_retry_timeout: settings.startup_retry_timeout,
            startup_retry_interval: settings.startup_retry_interval,
            refresh_interval: settings.refresh_interval,
            refresh_retry_interval: settings.refresh_retry_interval,
            stale_keys_ttl: settings.stale_keys_ttl,
        }
    }

    async fn load_jwks(&self) -> Result<JwkSet> {
        let jwks = self.provider.load().await?;

        if jwks.keys.is_empty() {
            return Err(Error::JwksLoadFailed(anyhow::anyhow!(
                "JWKS contains no keys"
            )));
        }

        Ok(jwks)
    }

    async fn replace_keys(&self, jwks: JwkSet) {
        let keys = VerificationKeySet::new(jwks);
        let key_count = keys.len();

        self.key_store.replace(keys).await;
        *self.last_loaded_at.write().await = Some(Instant::now());
        *self.status.write().await = HealthStatus::Ok;

        debug!(key_count, "JWKS loaded successfully");
    }

    async fn handle_refresh_failure(&self) {
        // Safe because refresh and refresh-failure handling are executed sequentially
        // by a single refresh loop.
        let Some(loaded_at) = *self.last_loaded_at.read().await else {
            return;
        };

        if loaded_at.elapsed() >= self.stale_keys_ttl {
            self.key_store.clear().await;
            *self.last_loaded_at.write().await = None;
            *self.status.write().await = HealthStatus::Unhealthy;

            warn!(
               stale_keys_ttl = ?self.stale_keys_ttl,
               "stale verification keys cleared"
            );
        } else {
            *self.status.write().await = HealthStatus::Degraded;
        }
    }

    // During initialization, retry loading the JWKS at startup_retry_interval
    // until startup_retry_timeout is reached.
    async fn initialize(&self) -> Result<()> {
        let deadline = Instant::now() + self.startup_retry_timeout;

        loop {
            match self.load_jwks().await {
                Ok(jwks) => {
                    self.replace_keys(jwks).await;
                    return Ok(());
                }

                Err(error) => {
                    let now = Instant::now();
                    if now >= deadline {
                        error!(
                            ?error,
                            startup_retry_timeout = ?self.startup_retry_timeout,
                            "failed to initialize JWKS"
                        );
                        return Err(error);
                    }

                    let retry_in = self
                        .startup_retry_interval
                        .min(deadline.saturating_duration_since(now));
                    warn!(?error, ?retry_in, "failed to load JWKS, retrying");
                    tokio::time::sleep(retry_in).await;
                }
            }
        }
    }

    async fn refresh(&self) -> Result<()> {
        let jwks = self.load_jwks().await?;

        self.replace_keys(jwks).await;

        Ok(())
    }

    // Refresh the JWKS every refresh_interval.
    // On failure, retry every refresh_retry_interval.
    // Clear the current keys once stale_keys_ttl is reached.
    fn spawn_refresh_loop(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(self.refresh_interval).await;

                loop {
                    match self.refresh().await {
                        Ok(()) => break,
                        Err(error) => {
                            warn!(?error, "failed to refresh JWKS");
                            self.handle_refresh_failure().await;
                            tokio::time::sleep(self.refresh_retry_interval).await;
                        }
                    }
                }
            }
        });
    }

    pub async fn start(
        settings: &Settings,
        key_store: Arc<VerificationKeyStore>,
    ) -> Result<Arc<Self>> {
        let service = Arc::new(Self::new(settings, key_store));

        service.initialize().await?;
        Arc::clone(&service).spawn_refresh_loop();

        Ok(service)
    }
}

#[async_trait]
impl HealthCheck for JwksService {
    async fn health(&self) -> HealthStatus {
        *self.status.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::jwks::MockJwksProvider;
    use mockall::Sequence;
    use serde_json::json;

    fn jwks(value: serde_json::Value) -> JwkSet {
        serde_json::from_value(value).unwrap()
    }

    fn empty_jwks() -> JwkSet {
        jwks(json!({"keys": []}))
    }

    fn valid_jwks() -> JwkSet {
        jwks(json!({
            "keys": [
                {
                    "kty": "RSA",
                    "kid": "key-1",
                    "alg": "RS256",
                    "n": "AQ",
                    "e": "AQ"
                }
            ]
        }))
    }

    fn settings() -> Settings {
        Settings {
            startup_retry_timeout: Duration::from_millis(50),
            startup_retry_interval: Duration::from_millis(10),
            refresh_interval: Duration::from_millis(50),
            refresh_retry_interval: Duration::from_millis(10),
            stale_keys_ttl: Duration::from_millis(100),
            ..Default::default()
        }
    }

    fn service(provider: MockJwksProvider) -> JwksService {
        let settings = settings();
        let key_store = Arc::new(VerificationKeyStore::new(VerificationKeySet::default()));
        let provider = Arc::new(provider);
        JwksService::with_provider(&settings, key_store, provider)
    }

    #[tokio::test]
    async fn health_returns_unhealthy_before_initialization() {
        let provider = MockJwksProvider::new();
        let service = service(provider);

        assert_eq!(service.health().await, HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn load_jwks_returns_jwks_when_not_empty() {
        let mut provider = MockJwksProvider::new();
        provider
            .expect_load()
            .once()
            .return_once(|| Ok(valid_jwks()));

        let service = service(provider);

        let result = service.load_jwks().await;

        let jwks = result.expect("JWKS should be loaded successfully");
        assert_eq!(jwks.keys.len(), 1);
    }

    #[tokio::test]
    async fn load_jwks_returns_error_when_jwks_is_empty() {
        let mut provider = MockJwksProvider::new();
        provider
            .expect_load()
            .once()
            .return_once(|| Ok(empty_jwks()));

        let service = service(provider);

        let result = service.load_jwks().await;

        assert!(matches!(result, Err(Error::JwksLoadFailed(_))));
    }

    #[tokio::test]
    async fn load_jwks_returns_provider_error_on_load_failure() {
        let mut provider = MockJwksProvider::new();
        provider
            .expect_load()
            .once()
            .return_once(|| Err(Error::JwksLoadFailed(anyhow::anyhow!("load failed"))));

        let service = service(provider);

        let result = service.load_jwks().await;

        assert!(matches!(result, Err(Error::JwksLoadFailed(_))));
    }

    #[tokio::test]
    async fn replace_keys_replaces_keys_updates_last_loaded_at_and_sets_status_to_ok() {
        let provider = MockJwksProvider::new();
        let service = service(provider);

        // Initial state
        assert!(service.key_store.current().await.is_empty());
        assert!(service.last_loaded_at.read().await.is_none());
        assert_eq!(service.health().await, HealthStatus::Unhealthy);

        // Replace with a non-empty JWKS
        service.replace_keys(valid_jwks()).await;
        assert_eq!(service.key_store.current().await.len(), 1);
        assert!(service.last_loaded_at.read().await.is_some());
        assert_eq!(service.health().await, HealthStatus::Ok);
        let first_loaded_at = service
            .last_loaded_at
            .read()
            .await
            .expect("last_loaded_at should be set");

        // Replace with a non-empty JWKS
        tokio::time::sleep(Duration::from_millis(1)).await;
        service.replace_keys(valid_jwks()).await;
        assert_eq!(service.key_store.current().await.len(), 1);
        assert!(service.last_loaded_at.read().await.is_some());
        assert_eq!(service.health().await, HealthStatus::Ok);
        let second_loaded_at = service
            .last_loaded_at
            .read()
            .await
            .expect("last_loaded_at should be set");
        assert!(second_loaded_at > first_loaded_at);
    }

    #[tokio::test]
    async fn handle_refresh_failure_keeps_fresh_keys_and_sets_status_to_degraded() {
        let provider = MockJwksProvider::new();
        let service = service(provider);

        service.replace_keys(valid_jwks()).await;

        assert_eq!(service.key_store.current().await.len(), 1);
        assert!(service.last_loaded_at.read().await.is_some());
        assert_eq!(service.health().await, HealthStatus::Ok);

        service.handle_refresh_failure().await;

        assert_eq!(service.key_store.current().await.len(), 1);
        assert!(service.last_loaded_at.read().await.is_some());
        assert_eq!(service.health().await, HealthStatus::Degraded);
    }

    #[tokio::test]
    async fn handle_refresh_failure_clears_stale_keys_and_sets_status_to_unhealthy() {
        let provider = MockJwksProvider::new();
        let service = service(provider);

        service.replace_keys(valid_jwks()).await;
        *service.last_loaded_at.write().await = Some(Instant::now() - service.stale_keys_ttl);

        assert_eq!(service.key_store.current().await.len(), 1);
        assert!(service.last_loaded_at.read().await.is_some());
        assert_eq!(service.health().await, HealthStatus::Ok);

        service.handle_refresh_failure().await;

        assert!(service.key_store.current().await.is_empty());
        assert!(service.last_loaded_at.read().await.is_none());
        assert_eq!(service.health().await, HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn initialize_succeeds_on_first_load() {
        let mut provider = MockJwksProvider::new();
        provider
            .expect_load()
            .once()
            .return_once(|| Ok(valid_jwks()));

        let service = service(provider);

        assert!(service.key_store.current().await.is_empty());
        assert!(service.last_loaded_at.read().await.is_none());
        assert_eq!(service.health().await, HealthStatus::Unhealthy);

        let result = service.initialize().await;

        assert!(result.is_ok());
        assert_eq!(service.key_store.current().await.len(), 1);
        assert!(service.last_loaded_at.read().await.is_some());
        assert_eq!(service.health().await, HealthStatus::Ok);
    }

    #[tokio::test]
    async fn initialize_retries_after_load_failure_and_succeeds() {
        let mut provider = MockJwksProvider::new();
        let mut sequence = Sequence::new();
        provider
            .expect_load()
            .once()
            .in_sequence(&mut sequence)
            .return_once(|| Err(Error::JwksLoadFailed(anyhow::anyhow!("load failed"))));
        provider
            .expect_load()
            .once()
            .in_sequence(&mut sequence)
            .return_once(|| Ok(valid_jwks()));

        let service = service(provider);

        assert!(service.key_store.current().await.is_empty());
        assert!(service.last_loaded_at.read().await.is_none());
        assert_eq!(service.health().await, HealthStatus::Unhealthy);

        let result = service.initialize().await;

        assert!(result.is_ok());
        assert_eq!(service.key_store.current().await.len(), 1);
        assert!(service.last_loaded_at.read().await.is_some());
        assert_eq!(service.health().await, HealthStatus::Ok);
    }

    #[tokio::test]
    async fn initialize_fails_when_retry_timeout_is_reached() {
        let mut provider = MockJwksProvider::new();
        provider
            .expect_load()
            .returning(|| Err(Error::JwksLoadFailed(anyhow::anyhow!("load failed"))));

        let service = service(provider);

        assert!(service.key_store.current().await.is_empty());
        assert!(service.last_loaded_at.read().await.is_none());
        assert_eq!(service.health().await, HealthStatus::Unhealthy);

        let result = service.initialize().await;

        assert!(matches!(result, Err(Error::JwksLoadFailed(_))));
        assert!(service.key_store.current().await.is_empty());
        assert!(service.last_loaded_at.read().await.is_none());
        assert_eq!(service.health().await, HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn refresh_replaces_keys_and_updates_last_loaded_at_on_success() {
        let mut provider = MockJwksProvider::new();
        provider
            .expect_load()
            .once()
            .return_once(|| Ok(valid_jwks()));

        let service = service(provider);

        assert!(service.key_store.current().await.is_empty());
        assert!(service.last_loaded_at.read().await.is_none());
        assert_eq!(service.health().await, HealthStatus::Unhealthy);

        let result = service.refresh().await;

        assert!(result.is_ok());
        assert_eq!(service.key_store.current().await.len(), 1);
        assert!(service.last_loaded_at.read().await.is_some());
        assert_eq!(service.health().await, HealthStatus::Ok);
    }

    #[tokio::test]
    async fn refresh_returns_error_and_keeps_existing_keys_on_failure() {
        let mut provider = MockJwksProvider::new();
        provider
            .expect_load()
            .once()
            .return_once(|| Err(Error::JwksLoadFailed(anyhow::anyhow!("load failed"))));

        let service = service(provider);

        assert!(service.key_store.current().await.is_empty());
        assert!(service.last_loaded_at.read().await.is_none());

        service.replace_keys(valid_jwks()).await;
        let loaded_at = service
            .last_loaded_at
            .read()
            .await
            .expect("last_loaded_at should be set");
        assert_eq!(service.key_store.current().await.len(), 1);

        let result = service.refresh().await;

        assert!(matches!(result, Err(Error::JwksLoadFailed(_))));
        assert_eq!(service.key_store.current().await.len(), 1);
        assert_eq!(
            service
                .last_loaded_at
                .read()
                .await
                .expect("last_loaded_at should still be set"),
            loaded_at
        );
    }

    #[tokio::test]
    async fn refresh_loop_refreshes_after_refresh_interval() {
        let mut provider = MockJwksProvider::new();
        provider
            .expect_load()
            .once()
            .return_once(|| Ok(valid_jwks()));

        let service = Arc::new(service(provider));

        assert!(service.key_store.current().await.is_empty());
        assert_eq!(service.health().await, HealthStatus::Unhealthy);

        Arc::clone(&service).spawn_refresh_loop();

        // 50 ms before first refresh + margin
        tokio::time::sleep(Duration::from_millis(75)).await;

        assert_eq!(service.key_store.current().await.len(), 1);
        assert!(service.last_loaded_at.read().await.is_some());
        assert_eq!(service.health().await, HealthStatus::Ok);
    }

    #[tokio::test]
    async fn refresh_loop_retries_after_refresh_failure() {
        let mut provider = MockJwksProvider::new();
        let mut sequence = Sequence::new();
        provider
            .expect_load()
            .once()
            .in_sequence(&mut sequence)
            .return_once(|| Err(Error::JwksLoadFailed(anyhow::anyhow!("load failed"))));
        provider
            .expect_load()
            .once()
            .in_sequence(&mut sequence)
            .return_once(|| Ok(valid_jwks()));

        let service = Arc::new(service(provider));

        assert!(service.key_store.current().await.is_empty());
        assert_eq!(service.health().await, HealthStatus::Unhealthy);

        Arc::clone(&service).spawn_refresh_loop();

        // 50 ms before first refresh + 10 ms before retry + margin
        tokio::time::sleep(Duration::from_millis(90)).await;

        assert_eq!(service.key_store.current().await.len(), 1);
        assert!(service.last_loaded_at.read().await.is_some());
        assert_eq!(service.health().await, HealthStatus::Ok);
    }

    #[tokio::test]
    async fn refresh_loop_clears_stale_keys_after_refresh_failure() {
        let mut provider = MockJwksProvider::new();
        provider
            .expect_load()
            .returning(|| Err(Error::JwksLoadFailed(anyhow::anyhow!("load failed"))));

        let service = Arc::new(service(provider));
        service.replace_keys(valid_jwks()).await;
        *service.last_loaded_at.write().await = Some(Instant::now() - service.stale_keys_ttl);

        assert_eq!(service.key_store.current().await.len(), 1);
        assert_eq!(service.health().await, HealthStatus::Ok);

        Arc::clone(&service).spawn_refresh_loop();

        // 50 ms before first refresh + margin
        tokio::time::sleep(Duration::from_millis(75)).await;

        assert!(service.key_store.current().await.is_empty());
        assert!(service.last_loaded_at.read().await.is_none());
        assert_eq!(service.health().await, HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn handle_refresh_failure_remains_unhealthy_when_no_keys_were_loaded() {
        let provider = MockJwksProvider::new();
        let service = service(provider);

        assert!(service.key_store.current().await.is_empty());
        assert!(service.last_loaded_at.read().await.is_none());
        assert_eq!(service.health().await, HealthStatus::Unhealthy);

        service.handle_refresh_failure().await;

        assert!(service.key_store.current().await.is_empty());
        assert!(service.last_loaded_at.read().await.is_none());
        assert_eq!(service.health().await, HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn refresh_loop_stops_retrying_after_successful_refresh() {
        let mut provider = MockJwksProvider::new();
        let mut sequence = Sequence::new();
        provider
            .expect_load()
            .once()
            .in_sequence(&mut sequence)
            .return_once(|| Err(Error::JwksLoadFailed(anyhow::anyhow!("load failed"))));
        provider
            .expect_load()
            .once()
            .in_sequence(&mut sequence)
            .return_once(|| Ok(valid_jwks()));

        let service = Arc::new(service(provider));

        Arc::clone(&service).spawn_refresh_loop();

        tokio::time::timeout(Duration::from_secs(1), async {
            while service.key_store.current().await.is_empty() {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .expect("refresh loop should load keys after retrying");

        assert_eq!(service.key_store.current().await.len(), 1);
        assert_eq!(service.health().await, HealthStatus::Ok);
    }

    #[tokio::test]
    async fn refresh_restores_ok_status_after_degradation() {
        let mut provider = MockJwksProvider::new();
        provider
            .expect_load()
            .once()
            .return_once(|| Ok(valid_jwks()));

        let service = service(provider);

        service.replace_keys(valid_jwks()).await;
        service.handle_refresh_failure().await;

        assert_eq!(service.health().await, HealthStatus::Degraded);

        let result = service.refresh().await;

        assert!(result.is_ok());
        assert_eq!(service.health().await, HealthStatus::Ok);
        assert_eq!(service.key_store.current().await.len(), 1);
        assert!(service.last_loaded_at.read().await.is_some());
    }
}
