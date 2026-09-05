use std::collections::BTreeMap;

use bkmt_desktop::{
    DesktopState, FileGenerationRequest, ReviewedFileInput, execute_file_generation,
    list_file_templates, plan_file_generation, save_user_template, set_file_generation_consent,
};

fn template_json() -> String {
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
    .to_string()
}

#[test]
fn desktop_template_catalog_includes_built_in_and_persisted_user_templates() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let state =
        DesktopState::new(data_root.path().to_path_buf()).expect("test data root must be absolute");

    let denied = save_user_template(&state, template_json())
        .expect_err("saving a user template without consent should be rejected");
    assert_eq!(denied.code, "consent_required");

    set_file_generation_consent(&state, true).expect("consent should be persisted");
    save_user_template(&state, template_json()).expect("user template should be persisted");
    let templates = list_file_templates(&state).expect("template catalog should load");

    assert!(
        templates
            .iter()
            .any(|template| template.source == "built_in")
    );
    assert!(templates.iter().any(|template| {
        template.id == "readme" && template.title == "README" && template.source == "user"
    }));
}

#[test]
fn desktop_command_plans_files_through_the_shared_core() {
    let destination = tempfile::tempdir().expect("destination should be created");
    let data_root = tempfile::tempdir().expect("data root should be created");
    let state =
        DesktopState::new(data_root.path().to_path_buf()).expect("test data root must be absolute");
    set_file_generation_consent(&state, true).expect("planning consent should be persisted");

    let plan = plan_file_generation(
        &state,
        FileGenerationRequest {
            template_json: template_json(),
            destination: destination.path().to_path_buf(),
            variables: BTreeMap::from([("name".to_string(), "Demo".to_string())]),
            overwrite: false,
        },
    )
    .expect("desktop adapter should return a plan");

    assert_eq!(plan.status, "planned");
    assert_eq!(plan.files[0].path, "README.md");
    assert_eq!(plan.files[0].action, "create");
    assert!(!destination.path().join("README.md").exists());
}

#[test]
fn desktop_planning_does_not_probe_paths_before_consent() {
    let destination = tempfile::tempdir().expect("destination should be created");
    let state = DesktopState::new(destination.path().join("data"))
        .expect("test data root must be absolute");

    let error = plan_file_generation(
        &state,
        FileGenerationRequest {
            template_json: template_json(),
            destination: destination.path().to_path_buf(),
            variables: BTreeMap::from([("name".to_string(), "Demo".to_string())]),
            overwrite: false,
        },
    )
    .expect_err("planning without consent should be rejected");

    assert_eq!(error.code, "consent_required");
}

#[test]
fn desktop_execution_is_authorized_in_rust_and_returns_each_file() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let destination = workspace.path().join("output");
    let state =
        DesktopState::new(workspace.path().join("data")).expect("test data root must be absolute");
    let request = || FileGenerationRequest {
        template_json: template_json(),
        destination: destination.clone(),
        variables: BTreeMap::from([("name".to_string(), "Demo".to_string())]),
        overwrite: false,
    };

    let reviewed = [ReviewedFileInput {
        path: "README.md".to_string(),
        action: "create".to_string(),
        target_revision: None,
    }];
    let denied = execute_file_generation(&state, request(), &reviewed)
        .expect_err("execution without consent should be rejected");
    assert_eq!(denied.code, "consent_required");
    assert!(!destination.join("README.md").exists());

    set_file_generation_consent(&state, true).expect("consent should be persisted");
    let report = execute_file_generation(&state, request(), &reviewed)
        .expect("authorized execution should generate files");

    assert_eq!(report.status, "complete");
    assert_eq!(report.files[0].path, "README.md");
    assert_eq!(report.files[0].outcome, "created");
    assert_eq!(
        std::fs::read_to_string(destination.join("README.md"))
            .expect("generated file should be readable"),
        "# Demo\n"
    );
}

#[test]
fn desktop_execution_rejects_a_destination_changed_after_preview() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let destination = workspace.path().join("output");
    std::fs::create_dir_all(&destination).expect("destination should be created");
    let state =
        DesktopState::new(workspace.path().join("data")).expect("test data root must be absolute");
    set_file_generation_consent(&state, true).expect("consent should be persisted");
    let request = FileGenerationRequest {
        template_json: template_json(),
        destination: destination.clone(),
        variables: BTreeMap::from([("name".to_string(), "Demo".to_string())]),
        overwrite: true,
    };
    let reviewed = [ReviewedFileInput {
        path: "README.md".to_string(),
        action: "create".to_string(),
        target_revision: None,
    }];
    std::fs::write(destination.join("README.md"), "arrived later\n")
        .expect("racing file should be created");

    let error = execute_file_generation(&state, request, &reviewed)
        .expect_err("changed plan should require another preview");

    assert_eq!(error.code, "plan_changed");
    assert_eq!(
        std::fs::read_to_string(destination.join("README.md")).expect("racing file should remain"),
        "arrived later\n"
    );
}

#[test]
fn desktop_execution_rejects_an_overwrite_target_changed_after_preview() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let destination = workspace.path().join("output");
    std::fs::create_dir_all(&destination).expect("destination should be created");
    std::fs::write(destination.join("README.md"), "original\n")
        .expect("original file should be created");
    let state =
        DesktopState::new(workspace.path().join("data")).expect("test data root must be absolute");
    set_file_generation_consent(&state, true).expect("consent should be persisted");
    let request = FileGenerationRequest {
        template_json: template_json(),
        destination: destination.clone(),
        variables: BTreeMap::from([("name".to_string(), "Demo".to_string())]),
        overwrite: true,
    };
    let plan = plan_file_generation(&state, request.clone()).expect("preview should succeed");
    std::fs::write(destination.join("README.md"), "changed after preview\n")
        .expect("target should change");
    let reviewed = plan
        .files
        .into_iter()
        .map(|file| ReviewedFileInput {
            path: file.path,
            action: file.action,
            target_revision: file.target_revision,
        })
        .collect::<Vec<_>>();

    let error = execute_file_generation(&state, request, &reviewed)
        .expect_err("changed overwrite target should require another preview");

    assert_eq!(error.code, "plan_changed");
    assert_eq!(
        std::fs::read_to_string(destination.join("README.md"))
            .expect("changed target should remain"),
        "changed after preview\n"
    );
}

#[test]
fn desktop_planning_rejects_empty_destinations_and_oversized_templates() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let state =
        DesktopState::new(data_root.path().to_path_buf()).expect("test data root must be absolute");
    set_file_generation_consent(&state, true).expect("consent should be persisted");
    let base = FileGenerationRequest {
        template_json: template_json(),
        destination: std::path::PathBuf::new(),
        variables: BTreeMap::from([("name".to_string(), "Demo".to_string())]),
        overwrite: false,
    };
    assert_eq!(
        plan_file_generation(&state, base.clone())
            .expect_err("empty destination should fail")
            .code,
        "invalid_destination"
    );
    let oversized = FileGenerationRequest {
        template_json: "x".repeat(1024 * 1024 + 1),
        destination: data_root.path().join("output"),
        ..base
    };
    assert_eq!(
        plan_file_generation(&state, oversized)
            .expect_err("oversized template should fail")
            .code,
        "user_template_too_large"
    );
}
