use crate::cli::Cli;
use crate::settings::Settings;

pub struct Application {
    pub settings: Settings,
}

impl Application {
    pub fn build(cli: &Cli) -> anyhow::Result<Self> {
        let settings = Settings::load(cli)?;

        Ok(Self { settings })
    }
}
