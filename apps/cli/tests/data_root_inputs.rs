use assert_cmd::Command;

#[test]
fn empty_environment_root_is_a_structured_input_error() {
    let output = Command::cargo_bin("bkmt")
        .unwrap()
        .env("BKMT_DATA_DIR", "")
        .args(["diagnostics", "data-dir", "--json"])
        .assert()
        .code(2)
        .get_output()
        .clone();
    assert!(output.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["code"], "invalid_data_root");
    assert!(error["message"].as_str().unwrap().contains("empty"));
}

#[cfg(unix)]
#[test]
fn json_diagnostics_rejects_non_utf8_roots_without_lossy_replacement() {
    use std::os::unix::ffi::OsStringExt;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join(std::ffi::OsString::from_vec(vec![0xff]));
    let output = Command::cargo_bin("bkmt")
        .unwrap()
        .arg("--data-dir")
        .arg(path)
        .args(["diagnostics", "data-dir", "--json"])
        .assert()
        .code(1)
        .get_output()
        .clone();
    assert!(output.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["code"], "data_root_not_unicode");
}
