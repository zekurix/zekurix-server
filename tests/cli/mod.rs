pub mod fixtures;

use assert_cmd::Command;

#[test]
fn should_display_version() {
    Command::cargo_bin("zekurix-server")
        .unwrap()
        .arg("--version")
        .assert()
        .success();
}

#[test]
fn should_fail_if_invalid_arg() {
    Command::cargo_bin("zekurix-server")
        .unwrap()
        .arg("--invalid")
        .assert()
        .failure();
}

#[test]
fn should_fail_if_explicit_config_does_not_exist() {
    let tmp_dir = tempfile::TempDir::new().unwrap();
    let path = tmp_dir.path().join("missing.toml");

    assert!(!path.exists());

    Command::cargo_bin("zekurix-server")
        .unwrap()
        .arg("--config")
        .arg(&path)
        .assert()
        .failure();
}

#[test]
fn should_succeed_dry_run_with_valid_configuration() {
    Command::cargo_bin("zekurix-server")
        .unwrap()
        .env_clear()
        .env("ZEKURIX_DISABLE_DOTENV", "true")
        .env("ZEKURIX_DATABASE__USERNAME", "postgres")
        .env("ZEKURIX_DATABASE__PASSWORD", "change-me")
        .env("ZEKURIX_AUTHENTICATION__ISSUER", "https://auth.example.com")
        .env("ZEKURIX_AUTHENTICATION__AUDIENCE", "zekurix")
        .env(
            "ZEKURIX_AUTHENTICATION__PROVIDER__JWKS_FILE",
            "tests/jwks.json",
        )
        .arg("--dry-run")
        .assert()
        .success();
}

#[test]
fn should_fail_dry_run_with_invalid_configuration() {
    Command::cargo_bin("zekurix-server")
        .unwrap()
        .env_clear()
        .env("ZEKURIX_DISABLE_DOTENV", "true")
        .env("ZEKURIX_DATABASE__PASSWORD", "change-me")
        .env("ZEKURIX_AUTHENTICATION__ISSUER", "https://auth.example.com")
        .env("ZEKURIX_AUTHENTICATION__AUDIENCE", "zekurix")
        .env(
            "ZEKURIX_AUTHENTICATION__PROVIDER__JWKS_FILE",
            "tests/jwks.json",
        )
        .arg("--dry-run")
        .assert()
        .failure();
}

#[test]
fn should_fail_dry_run_with_missing_secret() {
    Command::cargo_bin("zekurix-server")
        .unwrap()
        .env_clear()
        .env("ZEKURIX_DISABLE_DOTENV", "true")
        .env("ZEKURIX_DATABASE__USERNAME", "postgres")
        .env("ZEKURIX_AUTHENTICATION__ISSUER", "https://auth.example.com")
        .env("ZEKURIX_AUTHENTICATION__AUDIENCE", "zekurix")
        .env(
            "ZEKURIX_AUTHENTICATION__PROVIDER__JWKS_FILE",
            "tests/jwks.json",
        )
        .arg("--dry-run")
        .assert()
        .failure();
}
