use assert_cmd::Command;

#[test]
fn should_load_example_configuration() {
    Command::cargo_bin("zekurix-server")
        .unwrap()
        .env_clear()
        .env("ZEKURIX_DISABLE_DOTENV", "true")
        .env("ZEKURIX_DATABASE__PASSWORD", "change-me")
        .arg("--config")
        .arg("config/zekurix.example.toml")
        .arg("--dry-run")
        .assert()
        .success();
}

#[test]
fn should_load_dev_configuration() {
    Command::cargo_bin("zekurix-server")
        .unwrap()
        .env_clear()
        .env("ZEKURIX_DISABLE_DOTENV", "true")
        .env("ZEKURIX_DATABASE__USERNAME", "postgres")
        .env("ZEKURIX_DATABASE__PASSWORD", "change-me")
        .env("ZEKURIX_AUTHENTICATION__ISSUER", "https://auth.example.com")
        .env("ZEKURIX_AUTHENTICATION__AUDIENCE", "zekurix")
        .env("ZEKURIX_AUTHENTICATION__PROVIDER__JWKS_FILE", "tests/jwks.json")
        .arg("--config")
        .arg("config/zekurix.dev.toml")
        .arg("--dry-run")
        .assert()
        .success();
}

#[test]
fn should_load_test_configuration() {
    Command::cargo_bin("zekurix-server")
        .unwrap()
        .env_clear()
        .env("ZEKURIX_DISABLE_DOTENV", "true")
        .env("ZEKURIX_DATABASE__PASSWORD", "change-me")
        .arg("--config")
        .arg("config/zekurix.test.toml")
        .arg("--dry-run")
        .assert()
        .success();
}
