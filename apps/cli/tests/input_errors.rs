use assert_cmd::Command;

fn assert_json_error(arguments: &[&str], code: &str) {
    let result = Command::cargo_bin("bkmt")
        .unwrap()
        .args(arguments)
        .assert()
        .code(2)
        .get_output()
        .clone();
    assert!(result.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(error["code"], code);
}

#[test]
fn parser_failures_follow_the_json_error_contract() {
    assert_json_error(
        &[
            "file",
            "create",
            "--template",
            "@basic-readme",
            "--destination",
            ".",
            "--var",
            "invalid",
            "--json",
        ],
        "invalid_arguments",
    );
    assert_json_error(
        &[
            "spark",
            "analyze",
            "report",
            "--temperature",
            "nope",
            "--json",
        ],
        "invalid_arguments",
    );
    assert_json_error(&["file", "create", "--json"], "invalid_arguments");
}

#[test]
fn help_remains_human_readable_in_json_mode() {
    Command::cargo_bin("bkmt")
        .unwrap()
        .args(["file", "create", "--json", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Usage:"));
}

#[test]
fn empty_destination_is_rejected_before_loading_or_authorization() {
    assert_json_error(
        &[
            "file",
            "create",
            "--template",
            "missing.json",
            "--destination",
            "",
            "--dry-run",
            "--json",
        ],
        "invalid_arguments",
    );
}

#[test]
#[cfg(unix)]
fn invalid_template_catalog_parent_is_not_reported_as_success() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("tools"), "not a directory").unwrap();
    assert_json_error(
        &[
            "--data-dir",
            root.path().to_str().unwrap(),
            "file",
            "templates",
            "--json",
        ],
        "template_unreadable",
    );
}

#[test]
fn template_catalog_file_is_not_reported_as_a_directory() {
    let root = tempfile::tempdir().unwrap();
    let parent = root.path().join("tools/file-generator");
    std::fs::create_dir_all(&parent).unwrap();
    std::fs::write(parent.join("templates"), "not a directory").unwrap();
    assert_json_error(
        &[
            "--data-dir",
            root.path().to_str().unwrap(),
            "file",
            "templates",
            "--json",
        ],
        "template_unreadable",
    );
}
