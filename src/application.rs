use std::sync::Arc;

use anyhow::Result;
use tokio::signal;
use tracing::{debug, info};

use crate::cli::Cli;
use crate::router::build_router;
use crate::settings::Settings;

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    debug!("Waiting for shutdown signal (Ctrl+C or SIGTERM)");
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("Shutdown signal received");
}

pub struct Application {
    pub settings: Settings,
}

impl Application {
    pub fn build(cli: &Cli) -> Result<Arc<Self>> {
        let settings = Settings::load(cli)?;
        let application = Self { settings };

        Ok(Arc::new(application))
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        info!(
            version = env!("CARGO_PKG_VERSION"),
            "Starting Zekurix Server"
        );
        debug!(settings = ?self.settings, "Configuration loaded");

        let listener = tokio::net::TcpListener::bind(self.settings.server.socket_addr()?).await?;
        info!(address = %listener.local_addr()?, "Server listening");

        axum::serve(listener, build_router(self.clone()))
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        self.shutdown().await
    }

    async fn shutdown(self: Arc<Self>) -> Result<()> {
        info!("Shutting down Zekurix Server");
        Ok(())
    }
}
