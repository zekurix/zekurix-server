use std::fs;

use tempfile::NamedTempFile;

use zekurix_server::{cli::Cli, settings::Settings};

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
