use clap::Parser;

use zekurix_server::{
    application::Application,
    cli::Cli,
};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let application = Application::build(&cli)?;
    println!("{:?}", application.settings);

    Ok(())
}
