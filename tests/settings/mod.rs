use std::fs;
use std::time::Duration;

use tempfile::NamedTempFile;

use zekurix_server::cli::Cli;
use zekurix_server::settings::Settings;

#[test]
fn should_load_settings_from_default() {
    let cli = Cli::default();
    let settings = Settings::load(&cli).unwrap();

    assert_eq!(settings.server.host, "127.0.0.1");
    assert_eq!(settings.server.port, 8080);
}

#[test]
fn should_load_settings_from_toml() {
    let file = NamedTempFile::new().unwrap();

    fs::write(
        file.path(),
        r#"
        [server]
        host = "10.0.0.1"
        port = 1234
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
        acquire_timeout = "45s"
        idle_timeout = "15m"
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

#[test]
fn should_override_toml_with_cli_bind() {
    let file = NamedTempFile::new().unwrap();

    fs::write(
        file.path(),
        r#"
        [server]
        host = "10.0.0.1"
        port = 1234
        "#,
    )
    .unwrap();

    let cli = Cli {
        config: file.path().to_path_buf(),
        bind: Some("192.168.1.10:7777".parse().unwrap()),
        ..Default::default()
    };

    let settings = Settings::load(&cli).unwrap();

    assert_eq!(settings.server.host, "192.168.1.10");
    assert_eq!(settings.server.port, 7777);
}

#[test]
fn should_return_error_for_invalid_toml() {
    let file = NamedTempFile::new().unwrap();

    fs::write(
        file.path(),
        r#"
        [server
        host = "broken"
        "#,
    )
    .unwrap();

    let cli = Cli {
        config: file.path().to_path_buf(),
        ..Default::default()
    };

    assert!(Settings::load(&cli).is_err());
}

#[test]
fn should_return_error_for_unknown_fields_settings() {
    let file = NamedTempFile::new().unwrap();

    fs::write(
        file.path(),
        r#"
        [foo]
        "#,
    )
    .unwrap();

    let cli = Cli {
        config: file.path().to_path_buf(),
        ..Default::default()
    };

    assert!(Settings::load(&cli).is_err());
}

#[test]
fn should_return_error_for_unknown_fields_settings_logging() {
    let file = NamedTempFile::new().unwrap();

    fs::write(
        file.path(),
        r#"
        [logging]
        foo = "bar"
        "#,
    )
    .unwrap();

    let cli = Cli {
        config: file.path().to_path_buf(),
        ..Default::default()
    };

    assert!(Settings::load(&cli).is_err());
}

#[test]
fn should_return_error_for_unknown_fields_settings_server() {
    let file = NamedTempFile::new().unwrap();

    fs::write(
        file.path(),
        r#"
        [server]
        foo = "bar"
        "#,
    )
    .unwrap();

    let cli = Cli {
        config: file.path().to_path_buf(),
        ..Default::default()
    };

    assert!(Settings::load(&cli).is_err());
}

#[test]
fn should_return_error_for_unknown_fields_settings_database() {
    let file = NamedTempFile::new().unwrap();

    fs::write(
        file.path(),
        r#"
        [database]
        foo = "bar"
        "#,
    )
    .unwrap();

    let cli = Cli {
        config: file.path().to_path_buf(),
        ..Default::default()
    };

    assert!(Settings::load(&cli).is_err());
}
