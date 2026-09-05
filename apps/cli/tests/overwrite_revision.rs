#[cfg(unix)]
#[test]
fn overwrite_dry_run_rejects_inspection_outside_the_selected_destination() {
    let workspace = tempfile::tempdir().unwrap();
    let destination = workspace.path().join("output");
    let outside = workspace.path().join("outside");
    std::fs::create_dir(&destination).unwrap();
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("result.txt"), "outside").unwrap();
    std::os::unix::fs::symlink(&outside, destination.join("escape")).unwrap();
    let template = workspace.path().join("template.json");
    std::fs::write(&template, r#"{"schemaVersion":1,"id":"test","title":"test","variables":[],"files":[{"path":"escape/result.txt","content":"generated"}]}"#).unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_bkmt"))
        .arg("--data-dir")
        .arg(workspace.path().join("data"))
        .args(["file", "create", "--template"])
        .arg(&template)
        .arg("--destination")
        .arg(&destination)
        .args(["--force", "--dry-run", "--json"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(error["code"], "path_inspection_failed");
    assert_eq!(
        std::fs::read_to_string(outside.join("result.txt")).unwrap(),
        "outside"
    );
}
