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
