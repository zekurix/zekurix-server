use std::fs;

use tempfile::NamedTempFile;

use zekurix_server::cli::Cli;
use zekurix_server::settings::Settings;

#[test]
fn should_override_toml_with_cli_bind() {
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
        bind: Some("192.168.1.10:7777".parse().unwrap()),
        ..Default::default()
    };

    let settings = Settings::load(&cli).unwrap();

    assert_eq!(settings.server.host, "192.168.1.10");
    assert_eq!(settings.server.port, 7777);
}
