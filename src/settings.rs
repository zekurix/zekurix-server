use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};

use crate::cli::Cli;

const ENV_PREFIX: &str = "ZEKURIX_";

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
}

impl Settings {
    #[allow(clippy::result_large_err)]
    pub fn load(cli: &Cli) -> Result<Self, figment::Error> {
        // Merge settings in the following order:
        // Defaults < TOML < Environment Variables < CLI Overrides
        let mut settings: Self = Figment::new()
            .merge(Serialized::defaults(Settings::default()))
            .merge(Toml::file(&cli.config))
            .merge(Env::prefixed(ENV_PREFIX).split("__"))
            .extract()?;

        if let Some(bind) = cli.bind {
            settings.server.host = bind.ip().to_string();
            settings.server.port = bind.port();
        }

        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_use_default_server_settings() {
        let settings = ServerSettings::default();

        assert_eq!(settings.host, "127.0.0.1");
        assert_eq!(settings.port, 8080);
    }

    #[test]
    fn should_use_default_settings() {
        let settings = Settings::default();

        assert_eq!(settings.server.host, "127.0.0.1");
        assert_eq!(settings.server.port, 8080);
    }
}
