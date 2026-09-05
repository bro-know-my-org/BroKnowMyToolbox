use std::collections::BTreeMap;
use std::fs::{File, FileTimes};

use bkmt_desktop::{
    DesktopState, FileGenerationRequest, ReviewedFileInput, execute_file_generation,
    plan_file_generation, set_file_generation_consent,
};

#[test]
fn desktop_requires_a_new_preview_for_same_size_edits_with_unchanged_mtime() {
    let workspace = tempfile::tempdir().unwrap();
    let destination = workspace.path().join("output");
    std::fs::create_dir(&destination).unwrap();
    let target = destination.join("result.txt");
    std::fs::write(&target, "original").unwrap();
    let modified = std::fs::metadata(&target).unwrap().modified().unwrap();
    let state = DesktopState::new(workspace.path().join("data")).unwrap();
    set_file_generation_consent(&state, true).unwrap();
    let request = FileGenerationRequest {
        template_json: r#"{"schemaVersion":1,"id":"test","title":"test","variables":[],"files":[{"path":"result.txt","content":"generated"}]}"#.into(),
        destination,
        variables: BTreeMap::new(),
        overwrite: true,
    };
    let plan = plan_file_generation(&state, request.clone()).unwrap();
    std::fs::write(&target, "external").unwrap();
    File::options()
        .write(true)
        .open(&target)
        .unwrap()
        .set_times(FileTimes::new().set_modified(modified))
        .unwrap();
    let reviewed = plan
        .files
        .into_iter()
        .map(|file| ReviewedFileInput {
            path: file.path,
            action: file.action,
            target_revision: file.target_revision,
        })
        .collect::<Vec<_>>();
    let error = execute_file_generation(&state, request, &reviewed).unwrap_err();
    assert_eq!(error.code, "plan_changed");
    assert_eq!(std::fs::read_to_string(target).unwrap(), "external");
}
