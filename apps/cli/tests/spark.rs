use assert_cmd::Command;

fn allow_report_reading(data_root: &std::path::Path) {
    Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "--data-dir",
            data_root.to_str().expect("data path should be UTF-8"),
            "consent",
            "allow",
            "spark-analyzer",
            "filesystem:read",
        ])
        .assert()
        .success();
}

#[test]
fn spark_inspect_parses_a_local_text_report_through_bkmsa_core() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let data_root = workspace.path().join("data");
    let report = workspace.path().join("latest.log");
    std::fs::write(
        &report,
        "[Server thread/WARN]: Can't keep up! Is the server overloaded? Running 2000ms behind\n",
    )
    .expect("text report should be written");
    allow_report_reading(&data_root);

    let output = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "--data-dir",
            data_root.to_str().expect("data path should be UTF-8"),
            "spark",
            "inspect",
            report.to_str().expect("report path should be UTF-8"),
            "--text",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should contain JSON");
    assert_eq!(json["command"], "inspect");
    assert_eq!(json["kind"], "text");
    assert!(json["summary"].is_object());
}

#[test]
fn spark_lists_the_deterministic_tools_as_stable_json() {
    let output = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args(["spark", "tools", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should contain JSON");
    assert!(
        json.as_array()
            .is_some_and(|tools| { tools.iter().any(|tool| tool["name"] == "overview") })
    );
}

#[test]
fn spark_runs_a_deterministic_tool_against_a_local_report() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let data_root = workspace.path().join("data");
    let report = workspace.path().join("latest.log");
    std::fs::write(&report, "Can't keep up! Running 2000ms behind\n")
        .expect("text report should be written");
    allow_report_reading(&data_root);

    let output = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "--data-dir",
            data_root.to_str().expect("data path should be UTF-8"),
            "spark",
            "tool",
            report.to_str().expect("report path should be UTF-8"),
            "overview",
            "--text",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should contain JSON");
    assert_eq!(json["command"], "overview");
    assert_eq!(json["kind"], "text");
    assert!(json["result"].is_object());
}

#[test]
fn spark_ai_analysis_requires_network_consent_in_rust() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let data_root = workspace.path().join("data");
    let report = workspace.path().join("latest.log");
    std::fs::write(&report, "Can't keep up! Running 2000ms behind\n")
        .expect("text report should be written");
    allow_report_reading(&data_root);

    let error = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .env("BKMT_SPARK_API_KEY", "test-key")
        .args([
            "--data-dir",
            data_root.to_str().expect("data path should be UTF-8"),
            "spark",
            "analyze",
            report.to_str().expect("report path should be UTF-8"),
            "--text",
            "--json",
        ])
        .assert()
        .code(4)
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&error).expect("stderr should contain JSON");
    assert_eq!(json["code"], "consent_required");
    assert_eq!(json["details"]["capability"], "network:spark");
}

#[test]
fn spark_ai_analysis_requires_credential_consent_in_rust() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let data_root = workspace.path().join("data");
    let report = workspace.path().join("latest.log");
    std::fs::write(&report, "Can't keep up! Running 2000ms behind\n")
        .expect("text report should be written");
    allow_report_reading(&data_root);
    Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "--data-dir",
            data_root.to_str().expect("data path should be UTF-8"),
            "consent",
            "allow",
            "spark-analyzer",
            "network:spark",
        ])
        .assert()
        .success();

    let error = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .env("BKMT_SPARK_API_KEY", "test-key")
        .args([
            "--data-dir",
            data_root.to_str().expect("data path should be UTF-8"),
            "spark",
            "analyze",
            report.to_str().expect("report path should be UTF-8"),
            "--text",
            "--json",
        ])
        .assert()
        .code(4)
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&error).expect("stderr should contain JSON");
    assert_eq!(json["code"], "consent_required");
    assert_eq!(json["details"]["capability"], "credentials:ai");
}

#[test]
fn spark_ai_analysis_reports_a_missing_api_key_before_network_access() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let data_root = workspace.path().join("data");
    let report = workspace.path().join("latest.log");
    std::fs::write(&report, "Can't keep up! Running 2000ms behind\n")
        .expect("text report should be written");
    allow_report_reading(&data_root);
    for capability in ["network:spark", "credentials:ai"] {
        Command::cargo_bin("bkmt")
            .expect("bkmt binary should build")
            .args([
                "--data-dir",
                data_root.to_str().expect("data path should be UTF-8"),
                "consent",
                "allow",
                "spark-analyzer",
                capability,
            ])
            .assert()
            .success();
    }

    let error = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .env_remove("BKMT_SPARK_API_KEY")
        .env_remove("BKMSA_API_KEY")
        .args([
            "--data-dir",
            data_root.to_str().expect("data path should be UTF-8"),
            "spark",
            "analyze",
            report.to_str().expect("report path should be UTF-8"),
            "--text",
            "--json",
        ])
        .assert()
        .code(2)
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&error).expect("stderr should contain JSON");
    assert_eq!(json["code"], "missing_ai_credential");
}
