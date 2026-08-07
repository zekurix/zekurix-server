use clap::Parser;

use zekurix_server::{application::Application, cli::Cli, telemetry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let application = Application::build(&cli)?;

    telemetry::init(&application);
    application.run().await
}
