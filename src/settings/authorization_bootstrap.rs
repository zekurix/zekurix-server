use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::SuperAdminSettings;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct BootstrapSettings {
    pub super_admin: Option<SuperAdminSettings>,
}

impl BootstrapSettings {
    pub fn validate(&self, allowed_issuers: &[String]) -> Result<()> {
        match &self.super_admin {
            Some(super_admin) => super_admin.validate(allowed_issuers),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    fn validate_succeeds_when_super_admin_is_none() {
        let settings = BootstrapSettings { super_admin: None };

        assert!(settings.validate(&[]).is_ok());
    }

    #[test]
    fn validate_succeeds_when_super_admin_is_valid() {
        let settings = BootstrapSettings {
            super_admin: Some(SuperAdminSettings {
                issuer: "https://auth.example.com/my-issuer".into(),
                subject: "admin".into(),
            }),
        };
        let allowed_issuers = vec!["https://auth.example.com/my-issuer".into()];

        assert!(settings.validate(&allowed_issuers).is_ok());
    }

    #[test]
    fn validate_fails_when_super_admin_is_invalid() {
        let settings = BootstrapSettings {
            super_admin: Some(SuperAdminSettings {
                issuer: String::new(),
                subject: "admin".into(),
            }),
        };

        let result = settings.validate(&[]);
        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }
}
