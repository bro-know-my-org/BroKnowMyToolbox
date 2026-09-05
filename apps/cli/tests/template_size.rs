use assert_cmd::Command;

#[test]
fn cli_reads_a_one_mib_user_template_and_rejects_one_byte_more() {
    let workspace = tempfile::tempdir().unwrap();
    let data_root = workspace.path().join("data");
    let directory = data_root.join("tools/file-generator/templates");
    std::fs::create_dir_all(&directory).unwrap();
    let mut source = serde_json::json!({
        "schemaVersion": 1, "id": "boundary", "title": "Boundary", "variables": [],
        "files": [{ "path": "file.txt", "content": "test" }]
    })
    .to_string();
    source.push_str(&" ".repeat(1024 * 1024 - source.len()));
    let file = directory.join("boundary.json");
    std::fs::write(&file, &source).unwrap();
    let mut command = Command::cargo_bin("bkmt").unwrap();
    command
        .arg("--data-dir")
        .arg(&data_root)
        .args(["file", "create", "--template", "@boundary", "--destination"])
        .arg(workspace.path().join("output"))
        .args(["--dry-run", "--json"]);
    let output = command.assert().success().get_output().stdout.clone();
    let plan: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(plan["files"][0]["action"], "create");
    assert!(!workspace.path().join("output").exists());

    source.push(' ');
    std::fs::write(file, source).unwrap();
    let output = command.assert().code(2).get_output().stderr.clone();
    let error: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(error["code"], "user_template_too_large");
}
