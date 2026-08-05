mod cli;

use clap::Parser;

use crate::cli::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    println!("config: {}", cli.config.display());

    Ok(())
}
