use assert_cmd::Command;

#[test]
fn diagnostics_reports_the_environment_data_directory_as_json() {
    let data_root = tempfile::tempdir().expect("test data root should be created");
    let mut command = Command::cargo_bin("bkmt").expect("bkmt binary should build");

    let output = command
        .env("BKMT_DATA_DIR", data_root.path())
        .args(["diagnostics", "data-dir", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value =
        serde_json::from_slice(&output).expect("diagnostic output should be JSON");

    assert_eq!(value["path"], data_root.path().to_string_lossy().as_ref());
    assert_eq!(value["source"], "environment");
}

#[test]
fn portable_flag_reports_data_beside_the_executable() {
    let package = tempfile::tempdir().expect("portable package should be created");
    let executable = package
        .path()
        .join(if cfg!(windows) { "bkmt.exe" } else { "bkmt" });
    std::fs::copy(env!("CARGO_BIN_EXE_bkmt"), &executable)
        .expect("bkmt binary should be copied into the package");
    let mut command = Command::new(&executable);

    let output = command
        .env_remove("BKMT_DATA_DIR")
        .args(["--portable", "diagnostics", "data-dir", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value =
        serde_json::from_slice(&output).expect("diagnostic output should be JSON");
    let expected = package.path().join("data");

    assert_eq!(value["path"], expected.to_string_lossy().as_ref());
    assert_eq!(value["source"], "portable");
}

#[test]
fn relative_data_directory_is_a_structured_input_error() {
    let output = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "--data-dir",
            "relative-data",
            "diagnostics",
            "data-dir",
            "--json",
        ])
        .assert()
        .code(2)
        .get_output()
        .stderr
        .clone();
    let value: serde_json::Value =
        serde_json::from_slice(&output).expect("error output should be JSON");
    assert_eq!(value["code"], "invalid_data_root");
}
