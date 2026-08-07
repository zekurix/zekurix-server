use tracing::{debug, info};

use crate::cli::Cli;
use crate::router::build_router;
use crate::settings::Settings;

pub struct Application {
    pub settings: Settings,
}

impl Application {
    pub fn build(cli: &Cli) -> anyhow::Result<Self> {
        let settings = Settings::load(cli)?;

        Ok(Self { settings })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        info!(
            version = env!("CARGO_PKG_VERSION"),
            "Starting Zekurix Server"
        );
        debug!(settings = ?self.settings, "Configuration loaded");

        let listener = tokio::net::TcpListener::bind(self.settings.server.socket_addr()?).await?;
        info!(address = %listener.local_addr()?, "Server listening");

        axum::serve(listener, build_router()).await?;

        Ok(())
    }
}
