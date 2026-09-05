use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::args::FileCommand;
use crate::error::CliError;
use crate::runtime::resolve_data_root;

fn read_template_file(path: &std::path::Path, json: bool) -> Result<String, CliError> {
    file_generator::templates::read_template_file(path)
        .map_err(|error| CliError::input(error.code(), error.to_string(), json))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PlannedFileOutput<'a> {
    path: &'a std::path::Path,
    action: &'static str,
}

#[derive(serde::Serialize)]
struct GenerationPlanOutput<'a> {
    status: &'static str,
    files: Vec<PlannedFileOutput<'a>>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ExecutedFileOutput<'a> {
    path: &'a std::path::Path,
    outcome: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<&'a str>,
}

#[derive(serde::Serialize)]
struct ExecutionReportOutput<'a> {
    status: &'static str,
    files: Vec<ExecutedFileOutput<'a>>,
}

fn file_action_name(action: file_generator::FileAction) -> &'static str {
    match action {
        file_generator::FileAction::Create => "create",
        file_generator::FileAction::Overwrite => "overwrite",
        file_generator::FileAction::Conflict => "conflict",
    }
}

fn execution_status_name(status: file_generator::ExecutionStatus) -> &'static str {
    match status {
        file_generator::ExecutionStatus::Complete => "complete",
        file_generator::ExecutionStatus::Conflict => "conflict",
        file_generator::ExecutionStatus::PartialFailure => "partial_failure",
        file_generator::ExecutionStatus::Failed => "failed",
    }
}

fn file_outcome(outcome: &file_generator::FileOutcome) -> (&'static str, Option<&str>) {
    match outcome {
        file_generator::FileOutcome::Created => ("created", None),
        file_generator::FileOutcome::Overwritten => ("overwritten", None),
        file_generator::FileOutcome::SkippedConflict => ("skipped_conflict", None),
        file_generator::FileOutcome::Failed(error) => ("failed", Some(error)),
    }
}

fn templates(
    data_root: &toolbox_core::ResolvedDataRoot,
    json: bool,
) -> Result<file_generator::templates::TemplateCatalog, CliError> {
    let directory = data_root
        .path
        .join("tools")
        .join("file-generator")
        .join("templates");
    file_generator::templates::load_template_catalog(&directory)
        .map_err(|error| CliError::input(error.code(), error.to_string(), json))
}

fn load_template(
    template: &std::path::Path,
    data_dir: Option<PathBuf>,
    portable: bool,
    json: bool,
) -> Result<String, CliError> {
    let template_reference = template.to_string_lossy();
    let Some(template_id) = template_reference.strip_prefix('@') else {
        return read_template_file(template, json);
    };
    if !file_generator::valid_template_id(template_id) {
        return Err(CliError::input(
            "invalid_template_id",
            "template id must use 1-64 lowercase ASCII letters, digits, '-' or '_', excluding reserved Windows device names",
            json,
        ));
    }
    if let Some(template) = file_generator::built_in_templates()
        .map_err(|error| CliError::input("invalid_template", error.to_string(), json))?
        .into_iter()
        .find(|template| template.id == template_id)
    {
        return serde_json::to_string_pretty(&template).map_err(|error| {
            CliError::operation(
                "serialization_failed",
                format!("failed to serialize built-in template: {error}"),
                1,
                json,
            )
        });
    }
    let data_root = resolve_data_root(data_dir, portable, json)?;
    let path = data_root
        .path
        .join("tools")
        .join("file-generator")
        .join("templates")
        .join(format!("{template_id}.json"));
    let source = read_template_file(&path, json)?;
    let loaded = file_generator::TemplateDefinition::from_json(&source)
        .map_err(|error| CliError::input("invalid_template", error.to_string(), json))?;
    if loaded.id != template_id {
        return Err(CliError::input(
            "invalid_template_id",
            format!("user template filename must match id: {}", loaded.id),
            json,
        ));
    }
    Ok(source)
}

pub(crate) fn run(
    command: FileCommand,
    data_dir: Option<PathBuf>,
    portable: bool,
) -> Result<(), CliError> {
    match command {
        FileCommand::Templates { json } => {
            let data_root = resolve_data_root(data_dir, portable, json)?;
            let catalog = templates(&data_root, json)?;
            for warning in &catalog.warnings {
                if json {
                    eprintln!(
                        "{}",
                        serde_json::json!({
                            "level": "warning", "code": warning.code,
                            "fileName": warning.file_name, "message": warning.message,
                        })
                    );
                } else {
                    eprintln!(
                        "warning [{}] {}: {}",
                        warning.code, warning.file_name, warning.message
                    );
                }
            }
            let templates = catalog.templates;
            if json {
                println!(
                    "{}",
                    serde_json::to_string(
                        &templates
                            .iter()
                            .map(|entry| serde_json::json!({
                                "id": entry.definition.id,
                                "title": entry.definition.title,
                                "source": entry.source.as_str(),
                            }))
                            .collect::<Vec<_>>()
                    )
                    .map_err(|error| CliError::operation(
                        "serialization_failed",
                        format!("failed to serialize templates: {error}"),
                        1,
                        json
                    ))?
                );
            } else {
                for entry in templates {
                    println!(
                        "{}\t{}\t{}",
                        entry.definition.id,
                        entry.definition.title,
                        entry.source.as_str()
                    );
                }
            }
        }
        FileCommand::Create {
            template,
            destination,
            variables,
            dry_run,
            force,
            json,
        } => {
            let json_source = load_template(&template, data_dir.clone(), portable, json)?;
            let template = file_generator::TemplateDefinition::from_json(&json_source)
                .map_err(|error| CliError::input("invalid_template", error.to_string(), json))?;
            let generator = file_generator::FileGenerator::new();
            let plan = generator
                .plan_template(
                    &template,
                    &destination,
                    variables.into_iter().collect::<BTreeMap<_, _>>(),
                    force,
                )
                .map_err(|error| CliError::input(error.code(), error.to_string(), json))?;

            if dry_run {
                render_plan(&plan, json)?;
            } else {
                execute_plan(generator, &plan, &destination, data_dir, portable, json)?;
            }
        }
    }
    Ok(())
}

fn render_plan(plan: &file_generator::GenerationPlan, json: bool) -> Result<(), CliError> {
    if json {
        let output = GenerationPlanOutput {
            status: "planned",
            files: plan
                .files()
                .iter()
                .map(|file| PlannedFileOutput {
                    path: &file.relative_path,
                    action: file_action_name(file.action),
                })
                .collect(),
        };
        println!(
            "{}",
            serde_json::to_string(&output).map_err(|error| CliError::operation(
                "serialization_failed",
                format!("failed to serialize generation plan: {error}"),
                1,
                json
            ))?
        );
    } else {
        for file in plan.files() {
            println!(
                "{}\t{}",
                file_action_name(file.action),
                file.relative_path.display()
            );
        }
    }
    Ok(())
}

fn execute_plan(
    generator: file_generator::FileGenerator,
    plan: &file_generator::GenerationPlan,
    destination: &std::path::Path,
    data_dir: Option<PathBuf>,
    portable: bool,
    json: bool,
) -> Result<(), CliError> {
    let data_root = resolve_data_root(data_dir, portable, json)?;
    let authorizer = toolbox_core::CapabilityAuthorizer::new(toolbox_core::FileConsentStore::new(
        &data_root.path,
    ))
    .map_err(|error| CliError::operation("authorization_failed", error.to_string(), 1, json))?;
    authorizer
        .check(
            "file-generator",
            toolbox_core::CapabilityId::FilesystemWrite,
        )
        .map_err(|error| match error {
            toolbox_core::AuthorizationError::ConsentRequired => CliError::consent_required(json),
            toolbox_core::AuthorizationError::ConsentDenied => CliError::consent_denied(json),
            other => CliError::operation("authorization_failed", other.to_string(), 1, json),
        })?;
    std::fs::create_dir_all(destination).map_err(|error| {
        CliError::operation(
            "destination_unavailable",
            format!("failed to create destination: {error}"),
            1,
            json,
        )
    })?;
    let report = generator.execute(plan);
    let status = report.status();
    if json {
        let output = ExecutionReportOutput {
            status: execution_status_name(status),
            files: report
                .files()
                .iter()
                .map(|file| {
                    let (outcome, error) = file_outcome(&file.outcome);
                    ExecutedFileOutput {
                        path: &file.relative_path,
                        outcome,
                        error,
                    }
                })
                .collect(),
        };
        println!(
            "{}",
            serde_json::to_string(&output).map_err(|error| CliError::operation(
                "serialization_failed",
                format!("failed to serialize execution report: {error}"),
                1,
                json
            ))?
        );
    } else {
        for file in report.files() {
            let (outcome, error) = file_outcome(&file.outcome);
            if let Some(error) = error {
                println!("{outcome}\t{}\t{error}", file.relative_path.display());
            } else {
                println!("{outcome}\t{}", file.relative_path.display());
            }
        }
    }
    if let Some(error) = CliError::execution_status(status, json) {
        return Err(error);
    }
    Ok(())
}
