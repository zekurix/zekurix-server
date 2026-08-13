use std::sync::Arc;

use anyhow::Result;
use tokio::signal;
use tracing::{debug, info};

use crate::database::Database;
use crate::router::build_router;
use crate::secrets::Secrets;
use crate::settings::Settings;
use crate::users::Users;

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
    pub secrets: Secrets,
    pub database: Database,
    pub users: Users,
}

impl Application {
    pub async fn new(settings: Settings) -> Result<Arc<Self>> {
        info!(version = env!("CARGO_PKG_VERSION"), "Zekurix server");
        debug!(settings = ?settings, "Configuration loaded");

        let secrets = Secrets::load();
        let database = Database::connect(&settings.database, &secrets.database).await?;

        let application = Self {
            settings,
            secrets,
            database,
            users: Users::default(),
        };

        Ok(Arc::new(application))
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(self.settings.server.socket_addr()?).await?;
        info!(address = %listener.local_addr()?, "Server listening");

        axum::serve(listener, build_router(self.clone()))
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        self.shutdown().await
    }

    async fn shutdown(self: Arc<Self>) -> Result<()> {
        info!("Shutting down Zekurix server");
        Ok(())
    }
}
