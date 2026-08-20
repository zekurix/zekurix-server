use std::sync::Arc;

use anyhow::Result;
use tokio::signal;
use tracing::{debug, info};

use crate::postgres_database::PostgresDatabase;
use crate::routes;
use crate::secrets::Secrets;
use crate::settings::Settings;
use crate::user::postgres_repository::PostgresUserRepository;

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

    debug!("waiting for shutdown signal (Ctrl+C or SIGTERM)");
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("shutdown signal received");
}

pub struct Application {
    pub settings: Settings,
    pub secrets: Secrets,
    pub postgres_database: PostgresDatabase,
    pub postgres_user_repository: PostgresUserRepository,
}

impl Application {
    pub async fn new(settings: Settings) -> Result<Arc<Self>> {
        info!(version = env!("CARGO_PKG_VERSION"), "Zekurix server");
        debug!(settings = ?settings, "configuration loaded");

        let secrets = Secrets::load();
        let postgres_database =
            PostgresDatabase::init(&settings.database, &secrets.database).await?;
        let postgres_user_repository = PostgresUserRepository::new(postgres_database.pool.clone());

        let application = Self {
            settings,
            secrets,
            postgres_database,
            postgres_user_repository,
        };

        Ok(Arc::new(application))
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(self.settings.server.socket_addr()?).await?;
        info!(address = %listener.local_addr()?, "server listening");

        axum::serve(listener, routes::router(self.clone()))
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        self.shutdown().await
    }

    async fn shutdown(self: Arc<Self>) -> Result<()> {
        info!("shutting down Zekurix server");
        self.postgres_database.close().await;
        Ok(())
    }
}
