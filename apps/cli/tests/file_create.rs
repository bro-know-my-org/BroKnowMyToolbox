use assert_cmd::Command;

fn write_template_source() -> &'static str {
    r##"{
          "schemaVersion": 1,
          "id": "readme",
          "title": "README",
          "variables": [
            { "name": "name", "required": true, "default": null }
          ],
          "files": [
            { "path": "README.md", "content": "# {{name}}\n" }
          ]
        }"##
}

fn write_template(directory: &std::path::Path) -> std::path::PathBuf {
    let path = directory.join("template.json");
    std::fs::write(&path, write_template_source()).expect("template should be written");
    path
}

#[test]
fn file_template_catalog_lists_the_shared_built_in_template() {
    let data_root = tempfile::tempdir().expect("test data root should be created");
    let output = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "--data-dir",
            data_root
                .path()
                .to_str()
                .expect("data path should be UTF-8"),
            "file",
            "templates",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should contain JSON");
    assert_eq!(json[0]["id"], "basic-readme");
    assert_eq!(json[0]["source"], "built_in");
}

#[test]
fn file_create_can_use_a_shared_built_in_template_by_id() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let destination = workspace.path().join("output");

    let output = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "file",
            "create",
            "--template",
            "@basic-readme",
            "--destination",
            destination.to_str().expect("destination should be UTF-8"),
            "--var",
            "name=Demo",
            "--dry-run",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should contain JSON");
    assert_eq!(json["files"][0]["path"], "README.md");
}

#[test]
fn file_template_catalog_lists_user_templates_from_the_shared_data_root() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let data_root = workspace.path().join("data");
    let templates = data_root.join("tools/file-generator/templates");
    std::fs::create_dir_all(&templates).expect("template directory should be created");
    std::fs::write(templates.join("readme.json"), write_template_source())
        .expect("user template should be written");

    let output = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "--data-dir",
            data_root.to_str().expect("data root should be UTF-8"),
            "file",
            "templates",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should contain JSON");
    assert!(json.as_array().is_some_and(|templates| {
        templates
            .iter()
            .any(|template| template["id"] == "readme" && template["source"] == "user")
    }));
}

#[test]
fn dry_run_emits_a_json_plan_without_writing_files() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let template = write_template(workspace.path());
    let destination = workspace.path().join("output");

    let output = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "file",
            "create",
            "--template",
            template.to_str().expect("template path should be UTF-8"),
            "--destination",
            destination
                .to_str()
                .expect("destination path should be UTF-8"),
            "--var",
            "name=Demo",
            "--dry-run",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should contain JSON");
    assert_eq!(json["status"], "planned");
    assert_eq!(json["files"][0]["path"], "README.md");
    assert_eq!(json["files"][0]["action"], "create");
    assert!(!destination.join("README.md").exists());
}

#[test]
fn invalid_generation_input_emits_a_stable_json_error() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let template = write_template(workspace.path());
    let destination = workspace.path().join("output");

    let error = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "file",
            "create",
            "--template",
            template.to_str().expect("template path should be UTF-8"),
            "--destination",
            destination
                .to_str()
                .expect("destination path should be UTF-8"),
            "--dry-run",
            "--json",
        ])
        .assert()
        .code(2)
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&error).expect("stderr should contain JSON");
    assert_eq!(json["code"], "missing_variable");
    assert!(
        json["message"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
}

#[test]
fn cli_rejects_variables_not_declared_by_the_template() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let template = write_template(workspace.path());
    let destination = workspace.path().join("output");

    let error = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "file",
            "create",
            "--template",
            template.to_str().expect("template path should be UTF-8"),
            "--destination",
            destination
                .to_str()
                .expect("destination path should be UTF-8"),
            "--var",
            "name=Demo",
            "--var",
            "injected=bypass",
            "--dry-run",
            "--json",
        ])
        .assert()
        .code(2)
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&error).expect("stderr should contain JSON");
    assert_eq!(json["code"], "undeclared_variable");
}

#[test]
fn execution_requires_persisted_filesystem_write_consent() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let template = write_template(workspace.path());
    let destination = workspace.path().join("output");
    let data_root = workspace.path().join("data");

    let output = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "--data-dir",
            data_root.to_str().expect("data path should be UTF-8"),
            "file",
            "create",
            "--template",
            template.to_str().expect("template path should be UTF-8"),
            "--destination",
            destination
                .to_str()
                .expect("destination path should be UTF-8"),
            "--var",
            "name=Demo",
            "--json",
        ])
        .assert()
        .code(4)
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stderr should contain JSON");
    assert_eq!(json["code"], "consent_required");
    assert_eq!(json["details"]["toolId"], "file-generator");
    assert_eq!(json["details"]["capability"], "filesystem:write");
    assert!(!destination.join("README.md").exists());
}

#[test]
fn denied_filesystem_write_consent_blocks_execution() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let template = write_template(workspace.path());
    let destination = workspace.path().join("output");
    let data_root = workspace.path().join("data");

    Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "--data-dir",
            data_root.to_str().expect("data path should be UTF-8"),
            "consent",
            "deny",
            "file-generator",
            "filesystem:write",
        ])
        .assert()
        .success();

    let error = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "--data-dir",
            data_root.to_str().expect("data path should be UTF-8"),
            "file",
            "create",
            "--template",
            template.to_str().expect("template path should be UTF-8"),
            "--destination",
            destination
                .to_str()
                .expect("destination path should be UTF-8"),
            "--var",
            "name=Demo",
            "--json",
        ])
        .assert()
        .code(4)
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&error).expect("stderr should contain JSON");
    assert_eq!(json["code"], "consent_denied");
    assert!(!destination.join("README.md").exists());
}

#[test]
fn allowed_execution_writes_files_and_emits_a_json_report() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let template = write_template(workspace.path());
    let destination = workspace.path().join("output");
    let data_root = workspace.path().join("data");

    Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "--data-dir",
            data_root.to_str().expect("data path should be UTF-8"),
            "consent",
            "allow",
            "file-generator",
            "filesystem:write",
        ])
        .assert()
        .success();

    let output = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "--data-dir",
            data_root.to_str().expect("data path should be UTF-8"),
            "file",
            "create",
            "--template",
            template.to_str().expect("template path should be UTF-8"),
            "--destination",
            destination
                .to_str()
                .expect("destination path should be UTF-8"),
            "--var",
            "name=Demo",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should contain JSON");
    assert_eq!(json["status"], "complete");
    assert_eq!(json["files"][0]["path"], "README.md");
    assert_eq!(json["files"][0]["outcome"], "created");
    assert_eq!(
        std::fs::read_to_string(destination.join("README.md"))
            .expect("generated README should be readable"),
        "# Demo\n"
    );
}

#[test]
fn existing_files_are_reported_as_conflicts_without_force() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let template = write_template(workspace.path());
    let destination = workspace.path().join("output");
    let data_root = workspace.path().join("data");
    std::fs::create_dir_all(&destination).expect("destination should be created");
    std::fs::write(destination.join("README.md"), "keep\n")
        .expect("existing file should be written");

    Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "--data-dir",
            data_root.to_str().expect("data path should be UTF-8"),
            "consent",
            "allow",
            "file-generator",
            "filesystem:write",
        ])
        .assert()
        .success();

    let output = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "--data-dir",
            data_root.to_str().expect("data path should be UTF-8"),
            "file",
            "create",
            "--template",
            template.to_str().expect("template path should be UTF-8"),
            "--destination",
            destination
                .to_str()
                .expect("destination path should be UTF-8"),
            "--var",
            "name=Demo",
            "--json",
        ])
        .assert()
        .code(3)
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should contain JSON");
    assert_eq!(json["status"], "conflict");
    assert_eq!(json["files"][0]["outcome"], "skipped_conflict");
    assert_eq!(
        std::fs::read_to_string(destination.join("README.md"))
            .expect("existing file should remain readable"),
        "keep\n"
    );
}

#[test]
fn force_overwrites_existing_files() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let template = write_template(workspace.path());
    let destination = workspace.path().join("output");
    let data_root = workspace.path().join("data");
    std::fs::create_dir_all(&destination).expect("destination should be created");
    std::fs::write(destination.join("README.md"), "old\n")
        .expect("existing file should be written");

    Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "--data-dir",
            data_root.to_str().expect("data path should be UTF-8"),
            "consent",
            "allow",
            "file-generator",
            "filesystem:write",
        ])
        .assert()
        .success();

    let output = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "--data-dir",
            data_root.to_str().expect("data path should be UTF-8"),
            "file",
            "create",
            "--template",
            template.to_str().expect("template path should be UTF-8"),
            "--destination",
            destination
                .to_str()
                .expect("destination path should be UTF-8"),
            "--var",
            "name=Demo",
            "--force",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should contain JSON");
    assert_eq!(json["files"][0]["outcome"], "overwritten");
    assert_eq!(
        std::fs::read_to_string(destination.join("README.md"))
            .expect("overwritten file should be readable"),
        "# Demo\n"
    );
}

#[test]
fn file_directory_path_conflicts_are_rejected_before_execution() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let template = workspace.path().join("partial.json");
    std::fs::write(
        &template,
        r#"{
          "schemaVersion": 1,
          "id": "partial",
          "title": "Partial",
          "variables": [],
          "files": [
            { "path": "blocked", "content": "file" },
            { "path": "blocked/child.txt", "content": "child" }
          ]
        }"#,
    )
    .expect("template should be written");
    let destination = workspace.path().join("output");
    let data_root = workspace.path().join("data");

    Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "--data-dir",
            data_root.to_str().expect("data path should be UTF-8"),
            "consent",
            "allow",
            "file-generator",
            "filesystem:write",
        ])
        .assert()
        .success();

    let output = Command::cargo_bin("bkmt")
        .expect("bkmt binary should build")
        .args([
            "--data-dir",
            data_root.to_str().expect("data path should be UTF-8"),
            "file",
            "create",
            "--template",
            template.to_str().expect("template path should be UTF-8"),
            "--destination",
            destination
                .to_str()
                .expect("destination path should be UTF-8"),
            "--json",
        ])
        .assert()
        .code(2)
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should contain JSON");
    assert_eq!(json["code"], "path_conflict");
    assert!(!destination.exists());
}
