use std::collections::BTreeMap;
use std::sync::{Arc, Barrier};

use bkmt_file_generator::{FileGenerator, FileOutcome, GenerationRequest, TemplateFile};

#[test]
fn an_empty_directory_created_after_planning_is_not_replaced() {
    let destination = tempfile::tempdir().unwrap();
    let generator = FileGenerator::new();
    let plan = generator
        .plan(GenerationRequest {
            destination: destination.path().to_path_buf(),
            files: vec![TemplateFile {
                path: "result.txt".into(),
                content: "generated".into(),
            }],
            variables: BTreeMap::new(),
            overwrite: false,
        })
        .unwrap();
    std::fs::create_dir(destination.path().join("result.txt")).unwrap();
    let report = generator.execute(&plan);
    assert!(matches!(
        report.files()[0].outcome,
        FileOutcome::SkippedConflict | FileOutcome::Failed(_)
    ));
    assert!(destination.path().join("result.txt").is_dir());
    assert_eq!(std::fs::read_dir(destination.path()).unwrap().count(), 1);
}

#[test]
fn concurrent_creators_install_exactly_one_complete_file_without_clobbering() {
    let destination = tempfile::tempdir().unwrap();
    let plans = (0..8)
        .map(|writer| {
            FileGenerator::new()
                .plan(GenerationRequest {
                    destination: destination.path().to_path_buf(),
                    files: vec![TemplateFile {
                        path: "result.txt".into(),
                        content: format!("writer {writer}\n").repeat(4096),
                    }],
                    variables: BTreeMap::new(),
                    overwrite: false,
                })
                .unwrap()
        })
        .collect::<Vec<_>>();
    let barrier = Arc::new(Barrier::new(plans.len()));
    let outcomes = std::thread::scope(|scope| {
        let writers = plans
            .iter()
            .map(|plan| {
                let barrier = Arc::clone(&barrier);
                scope.spawn(move || {
                    barrier.wait();
                    FileGenerator::new().execute(plan).files()[0]
                        .outcome
                        .clone()
                })
            })
            .collect::<Vec<_>>();
        writers
            .into_iter()
            .map(|writer| writer.join().unwrap())
            .collect::<Vec<_>>()
    });
    let winners = outcomes
        .iter()
        .enumerate()
        .filter(|(_, outcome)| **outcome == FileOutcome::Created)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    assert_eq!(winners.len(), 1, "outcomes: {outcomes:?}");
    assert!(
        outcomes
            .iter()
            .all(|outcome| matches!(outcome, FileOutcome::Created | FileOutcome::SkippedConflict))
    );
    assert_eq!(
        std::fs::read_to_string(destination.path().join("result.txt")).unwrap(),
        plans[winners[0]].files()[0].content
    );
    assert_eq!(std::fs::read_dir(destination.path()).unwrap().count(), 1);
}
