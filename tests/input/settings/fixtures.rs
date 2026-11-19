use zekurix_server::settings::Settings;

use crate::input::cli::fixtures::test_cli;

pub(crate) fn test_settings() -> Settings {
    let cli = test_cli();

    Settings::load(&cli).expect("test configuration should be valid")
}
