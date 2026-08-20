use std::path::PathBuf;

use zekurix_server::cli::Cli;

pub fn cli_with_config(config: &str) -> Cli {
    let config = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("config")
        .join(config);

    Cli {
        config,
        ..Default::default()
    }
}

pub fn test_cli() -> Cli {
    cli_with_config("zekurix.test.toml")
}