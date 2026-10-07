use anyhow::Result;
use dotenvy::dotenv;
use tracing::{debug, info};

use zekurix_server::Application;
use zekurix_server::cli::Cli;
use zekurix_server::cli::ENV_DISABLE_DOTENV;
use zekurix_server::secrets::Secrets;
use zekurix_server::settings::Settings;
use zekurix_server::telemetry;

fn load_dotenv() {
    let dotenv_disabled = std::env::var(ENV_DISABLE_DOTENV)
        .ok()
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if !dotenv_disabled {
        dotenv().ok();
    }
}

fn dry_run(settings: Settings) -> Result<()> {
    info!(version = env!("CARGO_PKG_VERSION"), "Zekurix server");
    debug!(settings = ?settings, "configuration loaded");
    Secrets::load()?;
    info!("configuration and secrets are valid");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    load_dotenv();

    let cli = Cli::build()?;
    let settings = Settings::load(&cli)?;

    telemetry::init(&settings.logging);
    if cli.dry_run {
        dry_run(settings)
    } else {
        Application::new(settings).await?.run().await
    }
}
