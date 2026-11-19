use assert_cmd::Command;

#[test]
fn should_succeed_dry_run_with_valid_configuration() {
    Command::cargo_bin("zekurix-server")
        .unwrap()
        .env_clear()
        .env("ZEKURIX_DISABLE_DOTENV", "true")
        .env("ZEKURIX_DATABASE__USERNAME", "postgres")
        .env("ZEKURIX_DATABASE__PASSWORD", "change-me")
        .env(
            "ZEKURIX_AUTHENTICATION__ISSUERS",
            "[https://auth.example.com/my-issuer]",
        )
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
        .env(
            "ZEKURIX_AUTHENTICATION__ISSUERS",
            "[https://auth.example.com/my-issuer]",
        )
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
        .env(
            "ZEKURIX_AUTHENTICATION__ISSUERS",
            "[https://auth.example.com/my-issuer]",
        )
        .env("ZEKURIX_AUTHENTICATION__AUDIENCE", "zekurix")
        .env(
            "ZEKURIX_AUTHENTICATION__PROVIDER__JWKS_FILE",
            "tests/jwks.json",
        )
        .arg("--dry-run")
        .assert()
        .failure();
}
