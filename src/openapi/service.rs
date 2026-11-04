use axum::body::Bytes;

use crate::error::{Error, Result};
use crate::settings::AuthenticationSettings;

const OPENAPI_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/openapi/generated/openapi.min.json"
));

pub struct OpenApiService {
    document: Bytes,
}

impl OpenApiService {
    pub fn new(settings: &AuthenticationSettings) -> Result<Self> {
        let mut document: serde_json::Value =
            serde_json::from_str(OPENAPI_JSON).map_err(|err| Error::InternalError(err.into()))?;

        let issuers = document
            .pointer_mut("/components/schemas/Identity/properties/issuer/enum")
            .ok_or_else(|| {
                Error::InternalError(anyhow::anyhow!(
                    "OpenAPI identity issuer enumeration is missing"
                ))
            })?;

        *issuers = serde_json::json!(&settings.issuers);

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

    const ISSUER1: &str = "https://configured.example.com/my-issuer1";
    const ISSUER2: &str = "https://configured.example.com/my-issuer2";

    fn settings() -> AuthenticationSettings {
        AuthenticationSettings {
            issuers: vec![ISSUER1.to_owned(), ISSUER2.to_owned()],
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
            document.pointer("/components/schemas/Identity/properties/issuer/enum"),
            Some(&serde_json::json!([ISSUER1, ISSUER2]))
        );
    }
}
