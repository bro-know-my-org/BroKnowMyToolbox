use bkmt_desktop::{
    DesktopState, FileGenerationRequest, list_file_templates, plan_file_generation,
    save_user_template, set_file_generation_consent,
};

fn sized_template(size: usize) -> String {
    let mut source = serde_json::json!({
        "schemaVersion": 1, "id": "boundary", "title": "Boundary", "variables": [],
        "files": [{ "path": "file.txt", "content": "test" }]
    })
    .to_string();
    source.push_str(&" ".repeat(size - source.len()));
    source
}

#[test]
fn one_mib_template_round_trips_exactly_and_remains_plannable() {
    let workspace = tempfile::tempdir().unwrap();
    let state = DesktopState::new(workspace.path().join("data")).unwrap();
    set_file_generation_consent(&state, true).unwrap();
    let stored = workspace
        .path()
        .join("data/tools/file-generator/templates/boundary.json");
    for size in [1024 * 1024 - 1, 1024 * 1024] {
        let source = sized_template(size);
        save_user_template(&state, source.clone()).unwrap();
        assert_eq!(std::fs::read(&stored).unwrap(), source.as_bytes());
        let catalog = list_file_templates(&state).unwrap();
        let entry = catalog.iter().find(|entry| entry.id == "boundary").unwrap();
        assert_eq!(entry.template_json, source);
        let plan = plan_file_generation(
            &state,
            FileGenerationRequest {
                template_json: entry.template_json.clone(),
                destination: workspace.path().join("output"),
                variables: Default::default(),
                overwrite: false,
            },
        )
        .unwrap();
        assert_eq!(plan.files[0].action, "create");
    }
    let too_large = sized_template(1024 * 1024 + 1);
    assert_eq!(
        save_user_template(&state, too_large).unwrap_err().code,
        "user_template_too_large"
    );
    assert_eq!(
        std::fs::read_to_string(stored).unwrap(),
        sized_template(1024 * 1024)
    );
}
