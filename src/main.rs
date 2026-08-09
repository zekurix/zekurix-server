use anyhow::Result;

use zekurix_server::{application::Application, cli::Cli, telemetry};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::build()?;
    let application = Application::build(&cli)?;

    telemetry::init(&application);
    application.run().await?;
    application.shutdown().await
}
