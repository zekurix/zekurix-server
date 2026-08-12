pub mod database;

use anyhow::Result;
use figment::{Figment, providers::Env};
use serde::Deserialize;

const ENV_PREFIX: &str = "ZEKURIX_";

#[derive(Default, Deserialize)]
pub struct Secrets {
    pub database: database::Secrets,
}

impl Secrets {
    #[allow(clippy::result_large_err)]
    pub fn load() -> Result<Self> {
        let secrets = Figment::new()
            .merge(Env::prefixed(ENV_PREFIX).split("__"))
            .extract()?;

        Ok(secrets)
    }
}
