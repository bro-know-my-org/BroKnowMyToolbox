use bkmt_desktop::{
    DesktopState, FileGenerationRequest, ReviewedFileInput, execute_file_generation,
    plan_file_generation, set_file_generation_consent,
};
use std::collections::BTreeMap;

fn request(path: &str, destination: &std::path::Path) -> FileGenerationRequest {
    FileGenerationRequest {
        template_json: serde_json::json!({
            "schemaVersion": 1, "id": "paths", "title": "Paths", "variables": [],
            "files": [{ "path": path, "content": "replacement" }]
        })
        .to_string(),
        destination: destination.to_path_buf(),
        variables: BTreeMap::new(),
        overwrite: true,
    }
}

#[test]
fn desktop_rejects_nonportable_paths_with_the_shared_error_code() {
    let workspace = tempfile::tempdir().unwrap();
    let state = DesktopState::new(workspace.path().join("data")).unwrap();
    set_file_generation_consent(&state, true).unwrap();
    let destination = workspace.path().join("output");
    for path in ["CON.txt", "foo.", "foo ", ".", "dir/./file.txt"] {
        assert_eq!(
            plan_file_generation(&state, request(path, &destination))
                .unwrap_err()
                .code,
            "unsafe_path",
            "{path}"
        );
    }
    assert!(!destination.exists());
}

#[test]
fn desktop_overwrite_keeps_existing_directories_as_conflicts() {
    let workspace = tempfile::tempdir().unwrap();
    let state = DesktopState::new(workspace.path().join("data")).unwrap();
    set_file_generation_consent(&state, true).unwrap();
    let destination = workspace.path().join("output");
    std::fs::create_dir_all(destination.join("existing")).unwrap();
    std::fs::write(destination.join("existing/keep.txt"), "keep").unwrap();
    let input = request("existing", &destination);
    let plan = plan_file_generation(&state, input.clone()).unwrap();
    assert_eq!(plan.files[0].action, "conflict");
    let reviewed: Vec<_> = plan
        .files
        .into_iter()
        .map(|file| ReviewedFileInput {
            path: file.path,
            action: file.action,
            target_revision: file.target_revision,
        })
        .collect();
    let report = execute_file_generation(&state, input, &reviewed).unwrap();
    assert_eq!(report.status, "conflict");
    assert_eq!(report.files[0].outcome, "skipped_conflict");
    assert_eq!(
        std::fs::read_to_string(destination.join("existing/keep.txt")).unwrap(),
        "keep"
    );
}
