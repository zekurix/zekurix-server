use std::fs;
use tempfile::NamedTempFile;

use zekurix_server::settings::Settings;

use crate::cli::fixtures::cli_with_config;

#[test]
fn should_load_example_configuration() {
    let cli = cli_with_config("zekurix.example.toml");
    Settings::load(&cli).expect("example configuration should be valid");
}

#[test]
fn should_load_dev_configuration() {
    // `zekurix.dev.toml` is intentionally incomplete because the database username
    // is normally provided via environment variables. Append it here so the file
    // can be validated in isolation.
    let original = fs::read_to_string("config/zekurix.dev.toml").expect("should read dev config");
    let temp = NamedTempFile::new().expect("should create temp file");
    let content = format!("{original}\nusername = \"postgres\"\n");
    fs::write(temp.path(), content).expect("should write temp config");

    let cli = cli_with_config(temp.path().to_str().unwrap());
    Settings::load(&cli).expect("development configuration should be valid");
}

#[test]
fn should_load_test_configuration() {
    let cli = cli_with_config("zekurix.test.toml");
    Settings::load(&cli).expect("test configuration should be valid");
}
