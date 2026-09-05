use bkmt_desktop::{DesktopState, list_file_templates};

#[test]
fn desktop_returns_valid_templates_and_structured_warnings_together() {
    let root = tempfile::tempdir().unwrap();
    let state = DesktopState::new(root.path().to_path_buf()).unwrap();
    let directory = root.path().join("tools/file-generator/templates");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("broken.json"), "broken").unwrap();
    std::fs::write(
        directory.join("valid.json"),
        r#"{"schemaVersion":1,"id":"valid","title":"Valid","variables":[],"files":[]}"#,
    )
    .unwrap();
    let catalog = list_file_templates(&state).unwrap();
    assert_eq!(catalog.templates.len(), 2);
    assert_eq!(catalog.templates[0].source, "built_in");
    assert_eq!(catalog.templates[1].id, "valid");
    let wire = serde_json::to_value(&catalog).unwrap();
    assert_eq!(wire["warnings"][0]["fileName"], "broken.json");
    assert_eq!(wire["warnings"][0]["code"], "invalid_template");
    assert!(wire["templates"][1]["templateJson"].is_string());
    std::fs::remove_file(directory.join("broken.json")).unwrap();
    assert!(list_file_templates(&state).unwrap().warnings.is_empty());
}

#[test]
fn desktop_keeps_storage_errors_distinct_from_individual_template_warnings() {
    let root = tempfile::tempdir().unwrap();
    let state = DesktopState::new(root.path().to_path_buf()).unwrap();
    std::fs::write(root.path().join("tools"), "not a directory").unwrap();
    assert_eq!(
        list_file_templates(&state).unwrap_err().code,
        "template_storage_failed"
    );
}
