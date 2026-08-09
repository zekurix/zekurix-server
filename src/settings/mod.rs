pub mod logging;
pub mod server;

use anyhow::Result;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};

use crate::cli::Cli;

const ENV_PREFIX: &str = "ZEKURIX_";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    pub logging: logging::Settings,
    pub server: server::Settings,
}

impl Settings {
    fn merge(&mut self, cli: &Cli) {
        self.logging.merge(cli);
        self.server.merge(cli);
    }

    #[allow(clippy::result_large_err)]
    pub fn load(cli: &Cli) -> Result<Self> {
        // Merge settings in the following order:
        // Defaults < TOML < Environment Variables < CLI Overrides
        let mut settings: Self = Figment::new()
            .merge(Serialized::defaults(Settings::default()))
            .merge(Toml::file(&cli.config))
            .merge(Env::prefixed(ENV_PREFIX).split("__"))
            .extract()?;

        settings.merge(cli);
        Ok(settings)
    }
}
