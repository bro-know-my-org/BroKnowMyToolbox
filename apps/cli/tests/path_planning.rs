use assert_cmd::Command;

fn template(path: &str) -> String {
    serde_json::json!({
        "schemaVersion": 1, "id": "paths", "title": "Paths", "variables": [],
        "files": [{ "path": path, "content": "replacement" }]
    })
    .to_string()
}

#[test]
fn dry_run_rejects_nonportable_paths_with_the_shared_error_code() {
    let workspace = tempfile::tempdir().unwrap();
    let source = workspace.path().join("template.json");
    let destination = workspace.path().join("output");
    for path in ["CON.txt", "foo.", "foo ", ".", "dir/./file.txt"] {
        std::fs::write(&source, template(path)).unwrap();
        let output = Command::cargo_bin("bkmt")
            .unwrap()
            .args(["file", "create", "--template"])
            .arg(&source)
            .arg("--destination")
            .arg(&destination)
            .args(["--dry-run", "--json"])
            .assert()
            .code(2)
            .get_output()
            .clone();
        assert!(output.stdout.is_empty());
        let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(error["code"], "unsafe_path", "{path}");
    }
    assert!(!destination.exists());
}

#[test]
fn force_keeps_existing_directories_as_conflicts_in_plan_and_execution() {
    let workspace = tempfile::tempdir().unwrap();
    let source = workspace.path().join("template.json");
    let destination = workspace.path().join("output");
    let data_root = workspace.path().join("data");
    std::fs::write(&source, template("existing")).unwrap();
    std::fs::create_dir_all(destination.join("existing")).unwrap();
    std::fs::write(destination.join("existing/keep.txt"), "keep").unwrap();
    Command::cargo_bin("bkmt")
        .unwrap()
        .arg("--data-dir")
        .arg(&data_root)
        .args(["consent", "allow", "file-generator", "filesystem:write"])
        .assert()
        .success();

    for dry_run in [true, false] {
        let mut command = Command::cargo_bin("bkmt").unwrap();
        command
            .arg("--data-dir")
            .arg(&data_root)
            .args(["file", "create", "--template"])
            .arg(&source)
            .arg("--destination")
            .arg(&destination)
            .args(["--force", "--json"]);
        if dry_run {
            command.arg("--dry-run");
        }
        let output = command
            .assert()
            .code(if dry_run { 0 } else { 3 })
            .get_output()
            .clone();
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        if dry_run {
            assert_eq!(json["files"][0]["action"], "conflict");
        } else {
            assert_eq!(json["status"], "conflict");
            assert_eq!(json["files"][0]["outcome"], "skipped_conflict");
        }
    }
    assert_eq!(
        std::fs::read_to_string(destination.join("existing/keep.txt")).unwrap(),
        "keep"
    );
}
