use assert_cmd::Command;

fn source(id: &str) -> String {
    serde_json::json!({ "schemaVersion": 1, "id": id, "title": id, "variables": [], "files": [] })
        .to_string()
}

#[test]
fn listing_preserves_the_json_array_and_reports_damaged_templates_on_stderr() {
    let root = tempfile::tempdir().unwrap();
    let directory = root.path().join("tools/file-generator/templates");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("valid.json"), source("valid")).unwrap();
    std::fs::write(directory.join("broken.json"), "broken").unwrap();
    std::fs::write(directory.join("mismatch.json"), source("other")).unwrap();
    std::fs::write(directory.join("basic-readme.json"), source("basic-readme")).unwrap();
    std::fs::File::create(directory.join("large.json"))
        .unwrap()
        .set_len(1024 * 1024 + 1)
        .unwrap();
    let output = Command::cargo_bin("bkmt")
        .unwrap()
        .arg("--data-dir")
        .arg(root.path())
        .args(["file", "templates", "--json"])
        .assert()
        .success()
        .get_output()
        .clone();
    let templates: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(templates.as_array().unwrap().len(), 2);
    assert_eq!(templates[0]["source"], "built_in");
    assert_eq!(templates[1]["id"], "valid");
    let warnings: Vec<serde_json::Value> = std::str::from_utf8(&output.stderr)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(warnings.len(), 4);
    assert!(warnings.iter().all(|warning| warning["level"] == "warning"));
    assert_eq!(warnings[0]["fileName"], "basic-readme.json");
    assert_eq!(warnings[1]["code"], "invalid_template");
    assert_eq!(warnings[2]["code"], "user_template_too_large");
    assert_eq!(warnings[3]["code"], "invalid_template_id");
    let human = Command::cargo_bin("bkmt")
        .unwrap()
        .arg("--data-dir")
        .arg(root.path())
        .args(["file", "templates"])
        .assert()
        .success()
        .get_output()
        .clone();
    assert!(
        String::from_utf8(human.stdout)
            .unwrap()
            .contains("valid\tvalid\tuser")
    );
    assert!(
        String::from_utf8(human.stderr)
            .unwrap()
            .contains("warning [invalid_template] broken.json")
    );

    let error = Command::cargo_bin("bkmt")
        .unwrap()
        .arg("--data-dir")
        .arg(root.path())
        .args(["file", "create", "--template", "@broken", "--destination"])
        .arg(root.path().join("output"))
        .args(["--dry-run", "--json"])
        .assert()
        .code(2)
        .get_output()
        .stderr
        .clone();
    let error: serde_json::Value = serde_json::from_slice(&error).unwrap();
    assert_eq!(error["code"], "invalid_template");
    assert!(!root.path().join("output").exists());
    assert_eq!(
        std::fs::read_to_string(directory.join("broken.json")).unwrap(),
        "broken"
    );
}

#[cfg(unix)]
#[test]
fn fifo_templates_never_block_listing_or_explicit_loading() {
    use std::os::unix::ffi::OsStrExt;
    let root = tempfile::tempdir().unwrap();
    let directory = root.path().join("tools/file-generator/templates");
    std::fs::create_dir_all(&directory).unwrap();
    let fifo = directory.join("fifo.json");
    let path = std::ffi::CString::new(fifo.as_os_str().as_bytes()).unwrap();
    // SAFETY: path is a valid NUL-terminated pathname for this call.
    assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
    let output = Command::cargo_bin("bkmt")
        .unwrap()
        .timeout(std::time::Duration::from_secs(5))
        .arg("--data-dir")
        .arg(root.path())
        .args(["file", "templates", "--json"])
        .assert()
        .success()
        .get_output()
        .clone();
    let warning: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(warning["fileName"], "fifo.json");
    assert_eq!(warning["code"], "template_unreadable");
    let output = Command::cargo_bin("bkmt")
        .unwrap()
        .timeout(std::time::Duration::from_secs(5))
        .args(["file", "create", "--template"])
        .arg(fifo)
        .arg("--destination")
        .arg(root.path().join("output"))
        .args(["--dry-run", "--json"])
        .assert()
        .code(2)
        .get_output()
        .stderr
        .clone();
    let error: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(error["code"], "template_unreadable");
}
