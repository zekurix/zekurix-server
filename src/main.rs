use anyhow::Result;

use zekurix_server::application::Application;
use zekurix_server::cli::Cli;
use zekurix_server::settings::Settings;
use zekurix_server::telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::build()?;
    let settings = Settings::load(&cli)?;

    telemetry::init(&settings.logging);
    Application::new(settings).await?.run().await
}
