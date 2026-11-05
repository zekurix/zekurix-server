use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::BootstrapSettings;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct AuthorizationSettings {
    pub bootstrap: BootstrapSettings,
}

impl AuthorizationSettings {
    pub fn validate(&self, allowed_issuers: &[String]) -> Result<()> {
        self.bootstrap.validate(allowed_issuers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::settings::SuperAdminSettings;

    #[test]
    fn validate_succeeds_when_bootstrap_is_valid() {
        let settings = AuthorizationSettings {
            bootstrap: BootstrapSettings {
                super_admin: Some(SuperAdminSettings {
                    issuer: "https://auth.example.com/my-issuer".into(),
                    subject: "admin".into(),
                }),
            },
        };
        let allowed_issuers = vec!["https://auth.example.com/my-issuer".into()];

        assert!(settings.validate(&allowed_issuers).is_ok());
    }

    #[test]
    fn validate_fails_when_bootstrap_is_invalid() {
        let settings = AuthorizationSettings {
            bootstrap: BootstrapSettings {
                super_admin: Some(SuperAdminSettings {
                    issuer: String::new(),
                    subject: "admin".into(),
                }),
            },
        };

        let result = settings.validate(&[]);
        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }
}
