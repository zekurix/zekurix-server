use zekurix_server::settings::Settings;

use crate::cli::fixtures::cli_with_config;

#[test]
fn should_load_and_validate_example_configuration() {
    let cli = cli_with_config("zekurix.example.toml");

    let settings = Settings::load(&cli).expect("example configuration should be valid");

    let mut default = Settings::default();
    default.database.username = "postgres".to_string();

    assert_eq!(settings, default);
}

#[test]
fn should_load_and_validate_test_configuration() {
    let cli = cli_with_config("zekurix.test.toml");

    let settings = Settings::load(&cli).expect("test configuration should be valid");

    assert!(settings.database.migrate);
}
