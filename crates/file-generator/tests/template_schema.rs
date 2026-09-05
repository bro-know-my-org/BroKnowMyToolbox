use bkmt_file_generator::{TemplateDefinition, TemplateLoadError};
use std::collections::BTreeMap;

#[test]
fn template_definition_loads_the_versioned_json_schema() {
    let template = TemplateDefinition::from_json(
        r##"{
          "schemaVersion": 1,
          "id": "basic-project",
          "title": "Basic project",
          "variables": [
            { "name": "name", "required": true, "default": null }
          ],
          "files": [
            { "path": "README.md", "content": "# {{name}}\n" }
          ]
        }"##,
    )
    .expect("template definition should load");

    assert_eq!(template.schema_version, 1);
    assert_eq!(template.id, "basic-project");
    assert_eq!(template.variables[0].name, "name");
    assert!(template.variables[0].required);
    assert_eq!(template.files[0].path, "README.md");
}

#[test]
fn template_definition_rejects_unknown_schema_versions() {
    let error = TemplateDefinition::from_json(
        r#"{
          "schemaVersion": 99,
          "id": "future",
          "title": "Future template",
          "variables": [],
          "files": []
        }"#,
    )
    .expect_err("unknown schema version should be rejected");

    assert!(matches!(
        error,
        TemplateLoadError::UnsupportedSchemaVersion(99)
    ));
}

#[test]
fn template_definition_rejects_invalid_identity_and_variables() {
    let invalid_id = TemplateDefinition::from_json(
        r#"{
          "schemaVersion": 1,
          "id": "../escape",
          "title": "Unsafe",
          "variables": [],
          "files": []
        }"#,
    )
    .expect_err("unsafe template ids should be rejected by the shared core");
    assert!(matches!(invalid_id, TemplateLoadError::InvalidField(_)));

    let duplicate_variable = TemplateDefinition::from_json(
        r#"{
          "schemaVersion": 1,
          "id": "duplicate-variable",
          "title": "Duplicate variable",
          "variables": [
            { "name": "name", "required": true, "default": null },
            { "name": "name", "required": false, "default": "Demo" }
          ],
          "files": []
        }"#,
    )
    .expect_err("duplicate variables should be rejected");
    assert!(matches!(
        duplicate_variable,
        TemplateLoadError::DuplicateVariable(name) if name == "name"
    ));
}

#[test]
fn template_planning_applies_declared_variable_defaults() {
    let destination = tempfile::tempdir().expect("destination should be created");
    let template = TemplateDefinition::from_json(
        r##"{
          "schemaVersion": 1,
          "id": "readme",
          "title": "README",
          "variables": [
            { "name": "title", "required": false, "default": "My project" }
          ],
          "files": [
            { "path": "README.md", "content": "# {{title}}\n" }
          ]
        }"##,
    )
    .expect("template definition should load");

    let plan = bkmt_file_generator::FileGenerator::new()
        .plan_template(&template, destination.path(), BTreeMap::new(), false)
        .expect("defaulted template should be planned");

    assert_eq!(plan.files()[0].content, "# My project\n");
}

#[test]
fn template_planning_requires_every_declared_required_variable() {
    let destination = tempfile::tempdir().expect("destination should be created");
    let template = TemplateDefinition::from_json(
        r#"{
          "schemaVersion": 1,
          "id": "required-variable",
          "title": "Required variable",
          "variables": [
            { "name": "owner", "required": true, "default": null }
          ],
          "files": []
        }"#,
    )
    .expect("template definition should load");

    let result = bkmt_file_generator::FileGenerator::new().plan_template(
        &template,
        destination.path(),
        BTreeMap::new(),
        false,
    );

    assert_eq!(
        result,
        Err(bkmt_file_generator::GenerationError::MissingVariable(
            "owner".to_string()
        ))
    );
}

#[test]
fn template_planning_rejects_variables_that_are_not_declared_by_the_schema() {
    let error = TemplateDefinition::from_json(
        r#"{
          "schemaVersion": 1,
          "id": "declared-variables-only",
          "title": "Declared variables only",
          "variables": [],
          "files": [
            { "path": "README.md", "content": "{{injected}}" }
          ]
        }"#,
    )
    .expect_err("undeclared placeholders should fail while loading");

    assert!(
        matches!(error, TemplateLoadError::InvalidField(message) if message.contains("injected"))
    );
}

#[test]
fn optional_variables_without_defaults_render_as_empty_strings() {
    let destination = tempfile::tempdir().expect("destination should be created");
    let template = TemplateDefinition::from_json(
        r#"{
          "schemaVersion": 1,
          "id": "optional-variable",
          "title": "Optional variable",
          "variables": [
            { "name": "suffix", "required": false, "default": null }
          ],
          "files": [
            { "path": "README.md", "content": "Title{{suffix}}" }
          ]
        }"#,
    )
    .expect("optional template should load");

    let plan = bkmt_file_generator::FileGenerator::new()
        .plan_template(&template, destination.path(), BTreeMap::new(), false)
        .expect("omitted optional variable should be empty");

    assert_eq!(plan.files()[0].content, "Title");
}
