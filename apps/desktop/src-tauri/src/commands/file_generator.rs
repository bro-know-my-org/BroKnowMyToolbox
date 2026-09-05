use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::DesktopState;

const MAX_USER_TEMPLATE_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileGenerationRequest {
    pub template_json: String,
    pub destination: PathBuf,
    pub variables: BTreeMap<String, String>,
    pub overwrite: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedFileOutput {
    pub path: String,
    pub action: String,
    pub target_revision: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewedFileInput {
    pub path: String,
    pub action: String,
    pub target_revision: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FileGenerationPlanOutput {
    pub status: String,
    pub files: Vec<PlannedFileOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutedFileOutput {
    pub path: String,
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FileGenerationReportOutput {
    pub status: String,
    pub files: Vec<ExecutedFileOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileTemplateEntry {
    pub id: String,
    pub title: String,
    pub source: String,
    pub template_json: String,
}

fn template_directory(state: &DesktopState) -> PathBuf {
    state
        .data_root
        .join("tools")
        .join("file-generator")
        .join("templates")
}

fn template_entry(template_json: String, source: &str) -> Result<FileTemplateEntry, CommandError> {
    let template =
        file_generator::TemplateDefinition::from_json(&template_json).map_err(|error| {
            CommandError {
                code: "invalid_template".to_string(),
                message: error.to_string(),
            }
        })?;
    Ok(FileTemplateEntry {
        id: template.id,
        title: template.title,
        source: source.to_string(),
        template_json,
    })
}

pub fn list_file_templates(state: &DesktopState) -> Result<Vec<FileTemplateEntry>, CommandError> {
    let mut templates = vec![template_entry(
        file_generator::BASIC_README_TEMPLATE_JSON.to_string(),
        "built_in",
    )?];
    let directory = template_directory(state);
    if !directory.try_exists().map_err(|error| CommandError {
        code: "template_storage_failed".to_string(),
        message: error.to_string(),
    })? {
        return Ok(templates);
    }
    let entries = std::fs::read_dir(&directory).map_err(|error| CommandError {
        code: "template_storage_failed".to_string(),
        message: error.to_string(),
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| CommandError {
            code: "template_storage_failed".to_string(),
            message: error.to_string(),
        })?;
        let file_type = entry.file_type().map_err(|error| CommandError {
            code: "template_storage_failed".to_string(),
            message: error.to_string(),
        })?;
        if !file_type.is_file()
            || entry.path().extension().and_then(|value| value.to_str()) != Some("json")
        {
            continue;
        }
        let mut file = std::fs::File::open(entry.path()).map_err(|error| CommandError {
            code: "template_storage_failed".to_string(),
            message: error.to_string(),
        })?;
        let metadata = file.metadata().map_err(|error| CommandError {
            code: "template_storage_failed".to_string(),
            message: error.to_string(),
        })?;
        if metadata.len() > MAX_USER_TEMPLATE_BYTES {
            return Err(CommandError {
                code: "user_template_too_large".to_string(),
                message: format!("user template exceeds 1 MiB: {}", entry.path().display()),
            });
        }
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        std::io::Read::by_ref(&mut file)
            .take(MAX_USER_TEMPLATE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| CommandError {
                code: "template_storage_failed".to_string(),
                message: error.to_string(),
            })?;
        if bytes.len() as u64 > MAX_USER_TEMPLATE_BYTES {
            return Err(CommandError {
                code: "user_template_too_large".to_string(),
                message: format!("user template exceeds 1 MiB: {}", entry.path().display()),
            });
        }
        let template_json = String::from_utf8(bytes).map_err(|error| CommandError {
            code: "invalid_template".to_string(),
            message: error.to_string(),
        })?;
        let template = template_entry(template_json, "user")?;
        if entry.path().file_stem().and_then(|value| value.to_str()) != Some(&template.id) {
            return Err(CommandError {
                code: "invalid_template_id".to_string(),
                message: format!("user template filename must match id: {}", template.id),
            });
        }
        if templates.iter().any(|built_in| built_in.id == template.id) {
            return Err(CommandError {
                code: "invalid_template_id".to_string(),
                message: format!(
                    "user template id is reserved by a built-in template: {}",
                    template.id
                ),
            });
        }
        templates.push(template);
    }
    templates[1..].sort_by(|left, right| left.id.cmp(&right.id));
    Ok(templates)
}

pub fn save_user_template(
    state: &DesktopState,
    template_json: String,
) -> Result<FileTemplateEntry, CommandError> {
    authorize_filesystem_write(state)?;
    if template_json.len() as u64 > MAX_USER_TEMPLATE_BYTES {
        return Err(CommandError {
            code: "user_template_too_large".to_string(),
            message: "user template exceeds 1 MiB".to_string(),
        });
    }
    let template = template_entry(template_json, "user")?;
    if file_generator::built_in_templates()
        .map_err(|error| CommandError {
            code: "invalid_template".to_string(),
            message: error.to_string(),
        })?
        .iter()
        .any(|built_in| built_in.id == template.id)
    {
        return Err(CommandError {
            code: "invalid_template_id".to_string(),
            message: format!(
                "user template id is reserved by a built-in template: {}",
                template.id
            ),
        });
    }
    let directory = template_directory(state);
    std::fs::create_dir_all(&directory).map_err(|error| CommandError {
        code: "template_storage_failed".to_string(),
        message: error.to_string(),
    })?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(&directory).map_err(|error| CommandError {
            code: "template_storage_failed".to_string(),
            message: error.to_string(),
        })?;
    temporary
        .write_all(template.template_json.as_bytes())
        .and_then(|()| temporary.flush())
        .and_then(|()| temporary.as_file().sync_all())
        .map_err(|error| CommandError {
            code: "template_storage_failed".to_string(),
            message: error.to_string(),
        })?;
    temporary
        .persist(directory.join(format!("{}.json", template.id)))
        .map_err(|error| CommandError {
            code: "template_storage_failed".to_string(),
            message: error.error.to_string(),
        })?;
    #[cfg(unix)]
    std::fs::File::open(&directory)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| CommandError {
            code: "template_storage_failed".to_string(),
            message: error.to_string(),
        })?;
    Ok(template)
}

fn action_name(action: file_generator::FileAction) -> &'static str {
    match action {
        file_generator::FileAction::Create => "create",
        file_generator::FileAction::Overwrite => "overwrite",
        file_generator::FileAction::Conflict => "conflict",
    }
}

fn status_name(status: file_generator::ExecutionStatus) -> &'static str {
    match status {
        file_generator::ExecutionStatus::Complete => "complete",
        file_generator::ExecutionStatus::Conflict => "conflict",
        file_generator::ExecutionStatus::PartialFailure => "partial_failure",
        file_generator::ExecutionStatus::Failed => "failed",
    }
}

fn outcome(file: &file_generator::ExecutedFile) -> ExecutedFileOutput {
    let (name, error) = match &file.outcome {
        file_generator::FileOutcome::Created => ("created", None),
        file_generator::FileOutcome::Overwritten => ("overwritten", None),
        file_generator::FileOutcome::SkippedConflict => ("skipped_conflict", None),
        file_generator::FileOutcome::Failed(error) => ("failed", Some(error.clone())),
    };
    ExecutedFileOutput {
        path: file.relative_path.to_string_lossy().into_owned(),
        outcome: name.to_string(),
        error,
    }
}

fn map_authorization_error(error: toolbox_core::AuthorizationError) -> CommandError {
    let code = match error {
        toolbox_core::AuthorizationError::ConsentRequired => "consent_required",
        toolbox_core::AuthorizationError::ConsentDenied => "consent_denied",
        _ => "authorization_failed",
    };
    CommandError {
        code: code.to_string(),
        message: error.to_string(),
    }
}

fn authorize_filesystem_write(state: &DesktopState) -> Result<(), CommandError> {
    let authorizer = toolbox_core::CapabilityAuthorizer::new(toolbox_core::FileConsentStore::new(
        &state.data_root,
    ))
    .map_err(map_authorization_error)?;
    authorizer
        .check(
            "file-generator",
            toolbox_core::CapabilityId::FilesystemWrite,
        )
        .map_err(map_authorization_error)
}

pub fn plan_file_generation(
    state: &DesktopState,
    request: FileGenerationRequest,
) -> Result<FileGenerationPlanOutput, CommandError> {
    if request.destination.as_os_str().is_empty() {
        return Err(CommandError {
            code: "invalid_destination".to_string(),
            message: "destination must not be empty".to_string(),
        });
    }
    if request.template_json.len() as u64 > MAX_USER_TEMPLATE_BYTES {
        return Err(CommandError {
            code: "user_template_too_large".to_string(),
            message: "template exceeds 1 MiB".to_string(),
        });
    }
    authorize_filesystem_write(state)?;
    let template =
        file_generator::TemplateDefinition::from_json(&request.template_json).map_err(|error| {
            CommandError {
                code: "invalid_template".to_string(),
                message: error.to_string(),
            }
        })?;
    let plan = file_generator::FileGenerator::new()
        .plan_template(
            &template,
            request.destination,
            request.variables,
            request.overwrite,
        )
        .map_err(|error| CommandError {
            code: error.code().to_string(),
            message: error.to_string(),
        })?;

    Ok(FileGenerationPlanOutput {
        status: "planned".to_string(),
        files: plan
            .files()
            .iter()
            .map(|file| PlannedFileOutput {
                path: file.relative_path.to_string_lossy().into_owned(),
                action: action_name(file.action).to_string(),
                target_revision: file.target_revision.clone(),
            })
            .collect(),
    })
}

#[tauri::command(rename = "plan_file_generation")]
pub async fn plan_file_generation_command(
    state: tauri::State<'_, DesktopState>,
    request: FileGenerationRequest,
) -> Result<FileGenerationPlanOutput, CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || plan_file_generation(&state, request))
        .await
        .map_err(task_error)?
}

fn task_error(error: impl std::fmt::Display) -> CommandError {
    CommandError {
        code: "file_generation_task_failed".to_string(),
        message: error.to_string(),
    }
}

pub fn set_file_generation_consent(
    state: &DesktopState,
    allowed: bool,
) -> Result<(), CommandError> {
    let mut authorizer = toolbox_core::CapabilityAuthorizer::new(
        toolbox_core::FileConsentStore::new(&state.data_root),
    )
    .map_err(map_authorization_error)?;
    authorizer
        .record(
            "file-generator",
            toolbox_core::CapabilityId::FilesystemWrite,
            if allowed {
                toolbox_core::ConsentDecision::Allowed
            } else {
                toolbox_core::ConsentDecision::Denied
            },
        )
        .map_err(map_authorization_error)
}

pub fn execute_file_generation(
    state: &DesktopState,
    request: FileGenerationRequest,
    reviewed_files: &[ReviewedFileInput],
) -> Result<FileGenerationReportOutput, CommandError> {
    if request.destination.as_os_str().is_empty() {
        return Err(CommandError {
            code: "invalid_destination".to_string(),
            message: "destination must not be empty".to_string(),
        });
    }
    if request.template_json.len() as u64 > MAX_USER_TEMPLATE_BYTES {
        return Err(CommandError {
            code: "user_template_too_large".to_string(),
            message: "template exceeds 1 MiB".to_string(),
        });
    }
    authorize_filesystem_write(state)?;
    let template =
        file_generator::TemplateDefinition::from_json(&request.template_json).map_err(|error| {
            CommandError {
                code: "invalid_template".to_string(),
                message: error.to_string(),
            }
        })?;
    let generator = file_generator::FileGenerator::new();
    let plan = generator
        .plan_template(
            &template,
            &request.destination,
            request.variables,
            request.overwrite,
        )
        .map_err(|error| CommandError {
            code: error.code().to_string(),
            message: error.to_string(),
        })?;
    let current_files = plan
        .files()
        .iter()
        .map(|file| ReviewedFileInput {
            path: file.relative_path.to_string_lossy().into_owned(),
            action: action_name(file.action).to_string(),
            target_revision: file.target_revision.clone(),
        })
        .collect::<Vec<_>>();
    if current_files != reviewed_files {
        return Err(CommandError {
            code: "plan_changed".to_string(),
            message: "the destination changed after preview; review a new plan".to_string(),
        });
    }
    std::fs::create_dir_all(&request.destination).map_err(|error| CommandError {
        code: "destination_unavailable".to_string(),
        message: error.to_string(),
    })?;
    let report = generator.execute(&plan);
    Ok(FileGenerationReportOutput {
        status: status_name(report.status()).to_string(),
        files: report.files().iter().map(outcome).collect(),
    })
}

#[tauri::command]
pub async fn set_file_generation_consent_command(
    state: tauri::State<'_, DesktopState>,
    allowed: bool,
) -> Result<(), CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || set_file_generation_consent(&state, allowed))
        .await
        .map_err(task_error)?
}

#[tauri::command]
pub async fn execute_file_generation_command(
    state: tauri::State<'_, DesktopState>,
    request: FileGenerationRequest,
    reviewed_files: Vec<ReviewedFileInput>,
) -> Result<FileGenerationReportOutput, CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        execute_file_generation(&state, request, &reviewed_files)
    })
    .await
    .map_err(task_error)?
}

#[tauri::command]
pub async fn list_file_templates_command(
    state: tauri::State<'_, DesktopState>,
) -> Result<Vec<FileTemplateEntry>, CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || list_file_templates(&state))
        .await
        .map_err(task_error)?
}

#[tauri::command]
pub async fn save_user_template_command(
    state: tauri::State<'_, DesktopState>,
    template_json: String,
) -> Result<FileTemplateEntry, CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || save_user_template(&state, template_json))
        .await
        .map_err(task_error)?
}
