use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct SuperAdminSettings {
    pub issuer: String,
    pub subject: String,
}

impl SuperAdminSettings {
    pub fn validate(&self, allowed_issuers: &[String]) -> Result<()> {
        if self.issuer.trim().is_empty() {
            return Err(Error::InvalidSettings {
                setting: "authorization.bootstrap.super_admin.issuer".into(),
                reason: "cannot be empty".into(),
            });
        }

        if !allowed_issuers.contains(&self.issuer) {
            return Err(Error::InvalidSettings {
                setting: "authorization.bootstrap.super_admin.issuer".into(),
                reason: "is not present in authentication.issuers".into(),
            });
        }

        if self.subject.trim().is_empty() {
            return Err(Error::InvalidSettings {
                setting: "authorization.bootstrap.super_admin.subject".into(),
                reason: "cannot be empty".into(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_succeeds_when_settings_are_valid() {
        let settings = SuperAdminSettings {
            issuer: "https://auth.example.com/my-issuer".into(),
            subject: "admin".into(),
        };
        let allowed_issuers = vec!["https://auth.example.com/my-issuer".into()];

        assert!(settings.validate(&allowed_issuers).is_ok());
    }

    #[test]
    fn validate_fails_when_issuer_is_empty() {
        let settings = SuperAdminSettings {
            issuer: String::new(),
            subject: "admin".into(),
        };

        let result = settings.validate(&[]);
        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    #[test]
    fn validate_fails_when_issuer_is_whitespace_only() {
        let settings = SuperAdminSettings {
            issuer: "   ".into(),
            subject: "admin".into(),
        };

        let result = settings.validate(&[]);
        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    #[test]
    fn validate_fails_when_issuer_is_not_allowed() {
        let settings = SuperAdminSettings {
            issuer: "https://auth.example.com/my-issuer".into(),
            subject: "admin".into(),
        };
        let allowed_issuers = vec!["https://auth.example.com/invalid-issuer".into()];

        let result = settings.validate(&allowed_issuers);
        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    #[test]
    fn validate_fails_when_subject_is_empty() {
        let settings = SuperAdminSettings {
            issuer: "https://auth.example.com/my-issuer".into(),
            subject: String::new(),
        };
        let allowed_issuers = vec!["https://auth.example.com/my-issuer".into()];

        let result = settings.validate(&allowed_issuers);
        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    #[test]
    fn validate_fails_when_subject_is_whitespace_only() {
        let settings = SuperAdminSettings {
            issuer: "https://auth.example.com/my-issuer".into(),
            subject: "   ".into(),
        };
        let allowed_issuers = vec!["https://auth.example.com/my-issuer".into()];

        let result = settings.validate(&allowed_issuers);
        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }
}
