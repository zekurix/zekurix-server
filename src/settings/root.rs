use anyhow::Result;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};

use crate::cli::Cli;
use crate::secrets::Secrets;

use super::database;
use super::logging;
use super::server;

const ENV_PREFIX: &str = "ZEKURIX_";

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub logging: logging::Settings,
    pub server: server::Settings,
    pub database: database::Settings,
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
