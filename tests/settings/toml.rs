use zekurix_server::settings::Settings;

use crate::cli::fixtures::cli_with_config;

#[test]
fn should_load_and_validate_example_configuration() {
    let cli = cli_with_config("zekurix.example.toml");
    Settings::load(&cli).expect("example configuration should be valid");
}

#[test]
fn should_load_and_validate_test_configuration() {
    let cli = cli_with_config("zekurix.test.toml");
    Settings::load(&cli).expect("test configuration should be valid");
}
