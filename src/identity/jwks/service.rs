use std::sync::Arc;
use std::time::{Duration, Instant};

use jsonwebtoken::jwk::JwkSet;
use tokio::sync::RwLock;

use crate::error::Result;
use crate::settings::authentication_provider::Settings;

use super::{JwksProvider, VerificationKeySet, VerificationKeyStore, create_jwks_provider};

pub struct JwksService {
    provider: Arc<dyn JwksProvider>,
    store: Arc<VerificationKeyStore>,
    last_loaded: RwLock<Option<Instant>>,
    startup_retry_timeout: Duration,
    startup_retry_interval: Duration,
    refresh_interval: Duration,
    refresh_retry_interval: Duration,
    stale_keys_ttl: Duration,
}

impl JwksService {
    pub fn new(settings: &Settings, store: Arc<VerificationKeyStore>) -> Self {
        let provider = create_jwks_provider(settings);
        let last_loaded = RwLock::new(None);

        Self {
            provider,
            store,
            last_loaded,
            startup_retry_timeout: settings.startup_retry_timeout,
            startup_retry_interval: settings.startup_retry_interval,
            refresh_interval: settings.refresh_interval,
            refresh_retry_interval: settings.refresh_retry_interval,
            stale_keys_ttl: settings.stale_keys_ttl,
        }
    }

    async fn load_with_retry(&self, timeout: Duration, interval: Duration) -> Result<JwkSet> {
        let deadline = Instant::now() + timeout;

        loop {
            match self.provider.load().await {
                Ok(jwks) => return Ok(jwks),
                Err(error) => {
                    if Instant::now() >= deadline {
                        return Err(error);
                    }

                    tokio::time::sleep(interval).await;
                }
            }
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        let jwks = self
            .load_with_retry(self.startup_retry_timeout, self.startup_retry_interval)
            .await?;

        self.store.replace(VerificationKeySet::new(jwks)).await;
        let mut last_loaded = self.last_loaded.write().await;
        *last_loaded = Some(Instant::now());

        Ok(())
    }

    pub async fn refresh(&self) -> Result<()> {
        let jwks = self.provider.load().await?;
        self.store.replace(VerificationKeySet::new(jwks)).await;
        let mut last_loaded = self.last_loaded.write().await;
        *last_loaded = Some(Instant::now());

        Ok(())
    }

    async fn clear_stale_keys(&self) {
        let mut last_loaded = self.last_loaded.write().await;
        let is_stale = last_loaded
            .map(|loaded_at| Instant::now().duration_since(loaded_at) >= self.stale_keys_ttl)
            .unwrap_or(false);

        if is_stale {
            self.store.clear().await;
            *last_loaded = None;
        }
    }

    pub fn spawn_refresh_loop(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(self.refresh_interval).await;

                loop {
                    match self.refresh().await {
                        Ok(()) => break,
                        Err(error) => {
                            tracing::warn!(?error, "failed to refresh JWKS");
                            self.clear_stale_keys().await;
                            tokio::time::sleep(self.refresh_retry_interval).await;
                        }
                    }
                }
            }
        });
    }

    pub async fn start(settings: &Settings, store: Arc<VerificationKeyStore>) -> Result<Arc<Self>> {
        let mut service = Self::new(settings, store);
        service.initialize().await?;

        let service = Arc::new(service);
        service.clone().spawn_refresh_loop();

        Ok(service)
    }
}
