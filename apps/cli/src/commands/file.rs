use std::collections::BTreeMap;
use std::io::Read;
use std::path::PathBuf;

use crate::args::FileCommand;
use crate::error::CliError;
use crate::runtime::resolve_data_root;

const MAX_USER_TEMPLATE_BYTES: u64 = 1024 * 1024;

fn read_template_file(path: &std::path::Path, json: bool) -> Result<String, CliError> {
    let file = std::fs::File::open(path).map_err(|error| {
        CliError::input(
            "template_unreadable",
            format!("failed to read template: {error}"),
            json,
        )
    })?;
    let metadata = file
        .metadata()
        .map_err(|error| CliError::input("template_unreadable", error.to_string(), json))?;
    if !metadata.is_file() {
        return Err(CliError::input(
            "template_unreadable",
            "template must be a regular file",
            json,
        ));
    }
    if metadata.len() > MAX_USER_TEMPLATE_BYTES {
        return Err(CliError::input(
            "user_template_too_large",
            "user template exceeds 1 MiB",
            json,
        ));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_USER_TEMPLATE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| CliError::input("template_unreadable", error.to_string(), json))?;
    if bytes.len() as u64 > MAX_USER_TEMPLATE_BYTES {
        return Err(CliError::input(
            "user_template_too_large",
            "user template exceeds 1 MiB",
            json,
        ));
    }
    String::from_utf8(bytes)
        .map_err(|error| CliError::input("invalid_template", error.to_string(), json))
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
) -> Result<Vec<(file_generator::TemplateDefinition, &'static str)>, CliError> {
    let mut templates = file_generator::built_in_templates()
        .map_err(|error| CliError::input("invalid_template", error.to_string(), json))?
        .into_iter()
        .map(|template| (template, "built_in"))
        .collect::<Vec<_>>();
    let directory = data_root
        .path
        .join("tools")
        .join("file-generator")
        .join("templates");
    if !directory.exists() {
        return Ok(templates);
    }
    let entries = std::fs::read_dir(directory)
        .map_err(|error| CliError::input("template_unreadable", error.to_string(), json))?;
    let mut users = Vec::new();
    for entry in entries {
        let entry = entry
            .map_err(|error| CliError::input("template_unreadable", error.to_string(), json))?;
        let file_type = entry
            .file_type()
            .map_err(|error| CliError::input("template_unreadable", error.to_string(), json))?;
        if !file_type.is_file()
            || entry.path().extension().and_then(|value| value.to_str()) != Some("json")
        {
            continue;
        }
        let source = read_template_file(&entry.path(), json)?;
        let template = file_generator::TemplateDefinition::from_json(&source)
            .map_err(|error| CliError::input("invalid_template", error.to_string(), json))?;
        if entry.path().file_stem().and_then(|value| value.to_str()) != Some(&template.id) {
            return Err(CliError::input(
                "invalid_template_id",
                format!("user template filename must match id: {}", template.id),
                json,
            ));
        }
        if templates
            .iter()
            .any(|(built_in, _)| built_in.id == template.id)
        {
            return Err(CliError::input(
                "invalid_template_id",
                format!(
                    "user template id is reserved by a built-in template: {}",
                    template.id
                ),
                json,
            ));
        }
        users.push((template, "user"));
    }
    users.sort_by(|left, right| left.0.id.cmp(&right.0.id));
    templates.extend(users);
    Ok(templates)
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
            "template id must use 1-64 lowercase ASCII letters, digits, '-' or '_'",
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
            let templates = templates(&data_root, json)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string(
                        &templates
                            .iter()
                            .map(|(template, source)| serde_json::json!({
                                "id": template.id,
                                "title": template.title,
                                "source": source,
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
                for (template, source) in templates {
                    println!("{}\t{}\t{source}", template.id, template.title);
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
