use std::fs;
use std::time::Duration;

use tempfile::NamedTempFile;

use zekurix_server::cli::Cli;
use zekurix_server::settings::Settings;

#[test]
fn should_load_settings_from_toml() {
    let file = NamedTempFile::new().unwrap();

    fs::write(
        file.path(),
        r#"
        [server]
        host = "10.0.0.1"
        port = 1234

        [database]
        username = "postgres"

        [authentication]
        issuers = ["https://auth.example.com/my-issuer"]
        audience = "zekurix"

        [authentication.provider]
        jwks_file = "tests/jwks.json"
        "#,
    )
    .unwrap();

    let cli = Cli {
        config: file.path().to_path_buf(),
        ..Default::default()
    };

    let settings = Settings::load(&cli).unwrap();

    assert_eq!(settings.server.host, "10.0.0.1");
    assert_eq!(settings.server.port, 1234);
}

#[test]
fn should_load_humantime_settings_from_toml() {
    let file = NamedTempFile::new().unwrap();

    fs::write(
        file.path(),
        r#"
        [database]
        username = "postgres"
        acquire_timeout = "45s"
        idle_timeout = "15m"

        [authentication]
        issuers = ["https://auth.example.com/my-issuer"]
        audience = "zekurix"

        [authentication.provider]
        jwks_file = "tests/jwks.json"
        "#,
    )
    .unwrap();

    let cli = Cli {
        config: file.path().to_path_buf(),
        ..Default::default()
    };

    let settings = Settings::load(&cli).unwrap();

    assert_eq!(settings.database.acquire_timeout, Duration::from_secs(45));
    assert_eq!(settings.database.idle_timeout, Duration::from_secs(15 * 60));
    assert_eq!(settings.database.idle_timeout, Duration::from_mins(15));
}
