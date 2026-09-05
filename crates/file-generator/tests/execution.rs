use std::collections::BTreeMap;

use bkmt_file_generator::{
    ExecutionStatus, FileGenerator, FileOutcome, GenerationRequest, TemplateFile,
};

#[test]
fn execution_writes_the_plan_and_reports_each_file() {
    let destination = tempfile::tempdir().expect("destination should be created");
    let generator = FileGenerator::new();
    let plan = generator
        .plan(GenerationRequest {
            destination: destination.path().to_path_buf(),
            files: vec![
                TemplateFile {
                    path: "README.md".to_string(),
                    content: "# {{name}}\n".to_string(),
                },
                TemplateFile {
                    path: "config/app.toml".to_string(),
                    content: "name = \"{{name}}\"\n".to_string(),
                },
            ],
            variables: BTreeMap::from([("name".to_string(), "Demo".to_string())]),
            overwrite: false,
        })
        .expect("generation should be planned");

    let report = generator.execute(&plan);

    assert_eq!(
        report
            .files()
            .iter()
            .map(|file| &file.outcome)
            .collect::<Vec<_>>(),
        vec![&FileOutcome::Created, &FileOutcome::Created]
    );
    assert_eq!(
        std::fs::read_to_string(destination.path().join("README.md"))
            .expect("README should be written"),
        "# Demo\n"
    );
    assert_eq!(
        std::fs::read_to_string(destination.path().join("config/app.toml"))
            .expect("config should be written"),
        "name = \"Demo\"\n"
    );
}

#[test]
fn execution_skips_conflicts_without_changing_existing_files() {
    let destination = tempfile::tempdir().expect("destination should be created");
    std::fs::write(destination.path().join("README.md"), "keep me\n")
        .expect("existing file should be created");
    let generator = FileGenerator::new();
    let plan = generator
        .plan(GenerationRequest {
            destination: destination.path().to_path_buf(),
            files: vec![TemplateFile {
                path: "README.md".to_string(),
                content: "replace me\n".to_string(),
            }],
            variables: BTreeMap::new(),
            overwrite: false,
        })
        .expect("generation should be planned");

    let report = generator.execute(&plan);

    assert_eq!(report.status(), ExecutionStatus::Conflict);
    assert_eq!(report.files()[0].outcome, FileOutcome::SkippedConflict);
    assert_eq!(
        std::fs::read_to_string(destination.path().join("README.md"))
            .expect("existing file should remain readable"),
        "keep me\n"
    );
}

#[test]
fn execution_overwrites_existing_files_when_the_plan_allows_it() {
    let destination = tempfile::tempdir().expect("destination should be created");
    std::fs::write(destination.path().join("README.md"), "old\n")
        .expect("existing file should be created");
    let generator = FileGenerator::new();
    let plan = generator
        .plan(GenerationRequest {
            destination: destination.path().to_path_buf(),
            files: vec![TemplateFile {
                path: "README.md".to_string(),
                content: "new\n".to_string(),
            }],
            variables: BTreeMap::new(),
            overwrite: true,
        })
        .expect("generation should be planned");

    let report = generator.execute(&plan);

    assert_eq!(report.files()[0].outcome, FileOutcome::Overwritten);
    assert_eq!(
        std::fs::read_to_string(destination.path().join("README.md"))
            .expect("overwritten file should be readable"),
        "new\n"
    );
}

#[test]
fn execution_does_not_recreate_an_overwrite_target_deleted_after_planning() {
    let destination = tempfile::tempdir().expect("destination should be created");
    let target = destination.path().join("README.md");
    std::fs::write(&target, "old\n").expect("existing file should be created");
    let generator = FileGenerator::new();
    let plan = generator
        .plan(GenerationRequest {
            destination: destination.path().to_path_buf(),
            files: vec![TemplateFile {
                path: "README.md".to_string(),
                content: "new\n".to_string(),
            }],
            variables: BTreeMap::new(),
            overwrite: true,
        })
        .expect("generation should be planned");
    std::fs::remove_file(&target).expect("target should be removed after planning");

    let report = generator.execute(&plan);

    assert_eq!(report.status(), ExecutionStatus::Failed);
    assert!(!target.exists());
}

#[test]
fn execution_does_not_overwrite_a_file_created_after_planning() {
    let destination = tempfile::tempdir().expect("destination should be created");
    let generator = FileGenerator::new();
    let plan = generator
        .plan(GenerationRequest {
            destination: destination.path().to_path_buf(),
            files: vec![TemplateFile {
                path: "README.md".to_string(),
                content: "generated\n".to_string(),
            }],
            variables: BTreeMap::new(),
            overwrite: false,
        })
        .expect("generation should be planned");
    std::fs::write(destination.path().join("README.md"), "arrived later\n")
        .expect("racing file should be written");

    let report = generator.execute(&plan);

    assert_eq!(report.status(), ExecutionStatus::Conflict);
    assert_eq!(report.files()[0].outcome, FileOutcome::SkippedConflict);
    assert_eq!(
        std::fs::read_to_string(destination.path().join("README.md"))
            .expect("racing file should remain readable"),
        "arrived later\n"
    );
}

#[test]
fn execution_reports_partial_failure_without_hiding_successful_files() {
    let destination = tempfile::tempdir().expect("destination should be created");
    let generator = FileGenerator::new();
    let plan = generator
        .plan(GenerationRequest {
            destination: destination.path().to_path_buf(),
            files: vec![
                TemplateFile {
                    path: "created.txt".to_string(),
                    content: "created successfully\n".to_string(),
                },
                TemplateFile {
                    path: "blocked/child.txt".to_string(),
                    content: "cannot be created\n".to_string(),
                },
            ],
            variables: BTreeMap::new(),
            overwrite: false,
        })
        .expect("generation should be planned");
    std::fs::write(destination.path().join("blocked"), "blocks a directory\n")
        .expect("blocking file should be created");

    let report = generator.execute(&plan);

    assert_eq!(report.status(), ExecutionStatus::PartialFailure);
    assert_eq!(report.files()[0].outcome, FileOutcome::Created);
    assert!(matches!(report.files()[1].outcome, FileOutcome::Failed(_)));
}

#[cfg(unix)]
#[test]
fn execution_does_not_follow_a_symlink_outside_the_destination() {
    let destination = tempfile::tempdir().expect("destination should be created");
    let outside = tempfile::tempdir().expect("outside directory should be created");
    std::os::unix::fs::symlink(outside.path(), destination.path().join("escape"))
        .expect("symlink should be created");
    let generator = FileGenerator::new();
    let plan = generator
        .plan(GenerationRequest {
            destination: destination.path().to_path_buf(),
            files: vec![TemplateFile {
                path: "escape/payload.txt".to_string(),
                content: "must stay inside\n".to_string(),
            }],
            variables: BTreeMap::new(),
            overwrite: false,
        })
        .expect("the lexical path should be planned");

    let report = generator.execute(&plan);

    assert_eq!(report.status(), ExecutionStatus::Failed);
    assert!(matches!(report.files()[0].outcome, FileOutcome::Failed(_)));
    assert!(!outside.path().join("payload.txt").exists());
}
