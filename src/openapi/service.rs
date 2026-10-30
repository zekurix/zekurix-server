use axum::body::Bytes;

use crate::error::{Error, Result};
use crate::settings::authentication::Settings as AuthenticationSettings;

const OPENAPI_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/openapi/generated/openapi.min.json"
));

pub struct OpenApiService {
    document: Bytes,
}

impl OpenApiService {
    pub fn new(authentication_settings: &AuthenticationSettings) -> Result<Self> {
        let mut document: serde_json::Value =
            serde_json::from_str(OPENAPI_JSON).map_err(|err| Error::InternalError(err.into()))?;

        let issuer = document
            .pointer_mut("/components/schemas/Identity/properties/issuer/const")
            .ok_or_else(|| {
                Error::InternalError(anyhow::anyhow!("OpenAPI identity issuer const is missing"))
            })?;

        *issuer = serde_json::json!(authentication_settings.issuer);

        let document =
            serde_json::to_vec(&document).map_err(|err| Error::InternalError(err.into()))?;

        Ok(Self {
            document: Bytes::from(document),
        })
    }

    pub fn document(&self) -> Bytes {
        self.document.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ISSUER: &str = "https://configured.example.com/my-issuer";

    fn settings() -> AuthenticationSettings {
        AuthenticationSettings {
            issuer: ISSUER.to_owned(),
            ..Default::default()
        }
    }

    #[test]
    fn should_return_valid_openapi_document() {
        let service = OpenApiService::new(&settings()).unwrap();

        let document: serde_json::Value = serde_json::from_slice(&service.document()).unwrap();

        assert_eq!(document["openapi"], "3.1.0");
        assert!(document["paths"].is_object());
    }

    #[test]
    fn should_set_configured_identity_issuer() {
        let service = OpenApiService::new(&settings()).unwrap();

        let document: serde_json::Value = serde_json::from_slice(&service.document()).unwrap();

        assert_eq!(
            document.pointer("/components/schemas/Identity/properties/issuer/const"),
            Some(&serde_json::json!(ISSUER))
        );
    }
}
