use clap::Parser;

use zekurix_server::{application::Application, cli::Cli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    Application::build(&cli)?.run().await
}
