#[cfg(unix)]
#[test]
fn symlinked_consent_cannot_enable_cli_writes() {
    use assert_cmd::Command;
    let root = tempfile::tempdir().unwrap();
    let data = root.path().join("data");
    std::fs::create_dir_all(data.join("config")).unwrap();
    let outside = root.path().join("outside.json");
    std::fs::write(
        &outside,
        r#"{"schema_version":1,"decisions":{"file-generator":{"filesystem:write":"allowed"}}}"#,
    )
    .unwrap();
    std::os::unix::fs::symlink(&outside, data.join("config/consents.json")).unwrap();
    let destination = root.path().join("output");
    let output = Command::cargo_bin("bkmt")
        .unwrap()
        .arg("--data-dir")
        .arg(&data)
        .args([
            "file",
            "create",
            "--template",
            "@basic-readme",
            "--var",
            "name=Demo",
            "--destination",
        ])
        .arg(&destination)
        .arg("--json")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .code(1)
        .get_output()
        .clone();
    assert!(output.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["code"], "authorization_failed");
    assert!(!destination.exists());
}
