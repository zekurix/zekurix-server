use std::fs;

use tempfile::NamedTempFile;

use zekurix_server::cli::Cli;
use zekurix_server::settings::Settings;

#[test]
fn should_return_error_for_invalid_toml() {
    let file = NamedTempFile::new().unwrap();

    fs::write(
        file.path(),
        r#"
        [server
        host = "broken"

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

    assert!(Settings::load(&cli).is_err());
}

#[test]
fn should_return_error_for_unknown_fields_settings() {
    let file = NamedTempFile::new().unwrap();

    fs::write(
        file.path(),
        r#"
        [foo]

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

    assert!(Settings::load(&cli).is_err());
}
