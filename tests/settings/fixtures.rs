use zekurix_server::settings::Settings;

use crate::cli::fixtures::test_cli;

pub fn test_settings() -> Settings {
    let cli = test_cli();

    Settings::load(&cli).expect("test configuration should be valid")
}

#[test]
fn should_load_and_validate_test_configuration() {
    test_settings();
}
