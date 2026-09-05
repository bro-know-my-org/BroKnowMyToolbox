#[cfg(unix)]
#[test]
fn symlinked_consent_cannot_enable_desktop_writes() {
    use bkmt_desktop::{DesktopState, FileGenerationRequest, execute_file_generation};
    let root = tempfile::tempdir().unwrap();
    let data = root.path().join("data");
    std::fs::create_dir_all(data.join("config")).unwrap();
    let outside = root.path().join("outside.json");
    std::fs::write(
        &outside,
        r#"{"schema_version":1,"decisions":{"file-generator":{"filesystem:write":"allowed"}}}"#,
    )
    .unwrap();
    std::os::unix::fs::symlink(&outside, data.join("config/consents.json")).unwrap();
    let destination = root.path().join("output");
    let state = DesktopState::new(data);
    let error = execute_file_generation(
        &state,
        FileGenerationRequest {
            template_json: file_generator::BASIC_README_TEMPLATE_JSON.to_string(),
            destination: destination.clone(),
            variables: std::collections::BTreeMap::from([("name".into(), "Demo".into())]),
            overwrite: false,
        },
        &[],
    )
    .unwrap_err();
    assert_eq!(error.code, "authorization_failed");
    assert!(!destination.exists());
}
