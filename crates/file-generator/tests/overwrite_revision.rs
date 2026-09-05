use std::collections::BTreeMap;
use std::fs::{File, FileTimes};

use bkmt_file_generator::{ExecutionStatus, FileGenerator, GenerationRequest, TemplateFile};

#[test]
fn same_size_edit_with_restored_mtime_invalidates_the_overwrite_plan() {
    let destination = tempfile::tempdir().unwrap();
    let target = destination.path().join("result.txt");
    std::fs::write(&target, "original").unwrap();
    let modified = std::fs::metadata(&target).unwrap().modified().unwrap();
    let generator = FileGenerator::new();
    let request = GenerationRequest {
        destination: destination.path().to_path_buf(),
        files: vec![TemplateFile {
            path: "result.txt".into(),
            content: "generated".into(),
        }],
        variables: BTreeMap::new(),
        overwrite: true,
    };
    let plan = generator.plan(request.clone()).unwrap();
    std::fs::write(&target, "external").unwrap();
    File::options()
        .write(true)
        .open(&target)
        .unwrap()
        .set_times(FileTimes::new().set_modified(modified))
        .unwrap();
    let updated = generator.plan(request).unwrap();
    assert_ne!(
        plan.files()[0].target_revision,
        updated.files()[0].target_revision
    );
    assert_eq!(generator.execute(&plan).status(), ExecutionStatus::Failed);
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "external");
    assert_eq!(std::fs::read_dir(destination.path()).unwrap().count(), 1);
}

#[test]
fn replacing_a_file_with_identical_bytes_still_changes_its_identity() {
    let destination = tempfile::tempdir().unwrap();
    let target = destination.path().join("result.txt");
    std::fs::write(&target, "original").unwrap();
    let modified = std::fs::metadata(&target).unwrap().modified().unwrap();
    let generator = FileGenerator::new();
    let request = GenerationRequest {
        destination: destination.path().to_path_buf(),
        files: vec![TemplateFile {
            path: "result.txt".into(),
            content: "generated".into(),
        }],
        variables: BTreeMap::new(),
        overwrite: true,
    };
    let plan = generator.plan(request.clone()).unwrap();
    let replacement = destination.path().join("replacement.txt");
    std::fs::write(&replacement, "original").unwrap();
    File::options()
        .write(true)
        .open(&replacement)
        .unwrap()
        .set_times(FileTimes::new().set_modified(modified))
        .unwrap();
    std::fs::rename(&replacement, &target).unwrap();
    let updated = generator.plan(request).unwrap();
    assert_ne!(
        plan.files()[0].target_revision,
        updated.files()[0].target_revision
    );
    assert_eq!(generator.execute(&plan).status(), ExecutionStatus::Failed);
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "original");
}

#[cfg(unix)]
#[test]
fn overwrite_inspection_never_reads_through_a_link_outside_the_destination() {
    let destination = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("result.txt"), "outside").unwrap();
    std::os::unix::fs::symlink(outside.path(), destination.path().join("escape")).unwrap();
    let error = FileGenerator::new()
        .plan(GenerationRequest {
            destination: destination.path().to_path_buf(),
            files: vec![TemplateFile {
                path: "escape/result.txt".into(),
                content: "generated".into(),
            }],
            variables: BTreeMap::new(),
            overwrite: true,
        })
        .unwrap_err();
    assert_eq!(error.code(), "path_inspection_failed");
    assert_eq!(
        std::fs::read_to_string(outside.path().join("result.txt")).unwrap(),
        "outside"
    );
}
