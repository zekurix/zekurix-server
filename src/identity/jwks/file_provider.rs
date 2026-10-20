use std::path::PathBuf;

use async_trait::async_trait;
use jsonwebtoken::jwk::JwkSet;

use crate::error::{Error, Result};

use super::JwksProvider;

pub struct FileJwksProvider {
    path: PathBuf,
}

impl FileJwksProvider {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

#[async_trait]
impl JwksProvider for FileJwksProvider {
    async fn load(&self) -> Result<JwkSet> {
        let content = tokio::fs::read_to_string(&self.path)
            .await
            .map_err(|e| Error::JwksLoadFailed(e.into()))?;

        let jwks: JwkSet =
            serde_json::from_str(&content).map_err(|e| Error::JwksLoadFailed(e.into()))?;

        Ok(jwks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_load_jwks_from_file() {
        let provider = FileJwksProvider::new(PathBuf::from("tests/keys/authentication/jwks.json"));
        let jwks = provider.load().await.unwrap();

        assert_eq!(jwks.keys.len(), 1);
    }

    #[tokio::test]
    async fn should_fail_when_file_does_not_exist() {
        let provider = FileJwksProvider::new(PathBuf::from("does-not-exist.json"));
        let result = provider.load().await;

        assert!(matches!(result, Err(Error::JwksLoadFailed(_))));
    }

    #[tokio::test]
    async fn should_fail_when_file_contains_invalid_json() {
        let file = tempfile::NamedTempFile::new().unwrap();
        tokio::fs::write(file.path(), "not a json").await.unwrap();

        let provider = FileJwksProvider::new(file.path().to_path_buf());
        let result = provider.load().await;

        assert!(matches!(result, Err(Error::JwksLoadFailed(_))));
    }

    #[tokio::test]
    async fn should_fail_when_file_contains_invalid_jwks() {
        let file = tempfile::NamedTempFile::new().unwrap();
        tokio::fs::write(file.path(), r#"{"foo":"bar"}"#)
            .await
            .unwrap();

        let provider = FileJwksProvider::new(file.path().to_path_buf());
        let result = provider.load().await;

        assert!(matches!(result, Err(Error::JwksLoadFailed(_))));
    }
}
