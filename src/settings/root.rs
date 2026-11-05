use anyhow::Result;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};

use crate::cli::Cli;
use crate::secrets::Secrets;

use super::AuthenticationSettings;
use super::AuthorizationSettings;
use super::DatabaseSettings;
use super::LoggingSettings;
use super::ServerSettings;

const ENV_PREFIX: &str = "ZEKURIX_";

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub logging: LoggingSettings,
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub authentication: AuthenticationSettings,
    pub authorization: AuthorizationSettings,
}

impl Settings {
    fn strip_env_prefix(vars: Vec<&'static str>) -> Vec<&'static str> {
        vars.iter()
            .map(|env| env.strip_prefix(ENV_PREFIX).unwrap_or(env))
            .collect()
    }

    #[allow(clippy::result_large_err)]
    pub fn load(cli: &Cli) -> Result<Self> {
        // Merge settings in the following order:
        // Defaults < TOML < Environment Variables < CLI Overrides
        let mut settings: Self = Figment::new()
            .merge(Serialized::defaults(Settings::default()))
            .merge(Toml::file(&cli.config))
            .merge(
                Env::prefixed(ENV_PREFIX)
                    .ignore(&Self::strip_env_prefix(Cli::env_vars()))
                    .ignore(&Self::strip_env_prefix(Secrets::env_vars()))
                    .split("__"),
            )
            .extract()?;

        settings.merge(cli);
        settings.validate()?;
        Ok(settings)
    }

    fn merge(&mut self, cli: &Cli) {
        self.logging.merge(cli);
        self.server.merge(cli);
    }

    fn validate(&self) -> Result<()> {
        self.database.validate()?;
        self.authentication.validate()?;
        self.authorization.validate(&self.authentication.issuers)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_SETTINGS_TOML: &str = include_str!("../../config/zekurix.example.toml");

    fn valid_settings() -> Settings {
        Figment::new()
            .merge(Serialized::defaults(Settings::default()))
            .merge(Toml::string(VALID_SETTINGS_TOML))
            .extract()
            .expect("example settings should deserialize")
    }

    #[test]
    fn should_strip_env_prefix() {
        let vars = vec!["ZEKURIX_DATABASE__PASSWORD", "ZEKURIX_STORAGE__SECRET_KEY"];

        let result = Settings::strip_env_prefix(vars);

        assert_eq!(result, vec!["DATABASE__PASSWORD", "STORAGE__SECRET_KEY",]);
    }

    #[test]
    fn should_keep_env_var_when_prefix_is_missing() {
        let vars = vec!["DATABASE__PASSWORD"];

        let result = Settings::strip_env_prefix(vars);

        assert_eq!(result, vec!["DATABASE__PASSWORD",]);
    }

    #[test]
    fn example_settings_should_be_valid() {
        let settings = valid_settings();

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn should_reject_invalid_database_settings() {
        let mut settings = valid_settings();
        settings.database.min_connections = 6;
        settings.database.max_connections = 5;

        let result = settings.validate();

        assert!(result.is_err());
    }

    #[test]
    fn should_reject_invalid_authentication_settings() {
        let mut settings = valid_settings();
        settings.authentication.issuers.clear();

        let result = settings.validate();

        assert!(result.is_err());
    }

    #[test]
    fn should_reject_invalid_authorization_settings() {
        let mut settings = valid_settings();
        settings
            .authorization
            .bootstrap
            .super_admin
            .as_mut()
            .expect("example settings should contain a super administrator")
            .issuer = "https://auth.example.com/invalid-issuer".into();

        let result = settings.validate();

        assert!(result.is_err());
    }
}
