use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};

mod install;
pub mod templates;

const MAX_RENDERED_OUTPUT_BYTES: usize = 64 * 1024 * 1024;

pub const BASIC_README_TEMPLATE_JSON: &str = r##"{
  "schemaVersion": 1,
  "id": "basic-readme",
  "title": "README",
  "variables": [
    { "name": "name", "required": true, "default": null }
  ],
  "files": [
    { "path": "README.md", "content": "# {{name}}\n" }
  ]
}"##;

pub fn built_in_templates() -> Result<Vec<TemplateDefinition>, TemplateLoadError> {
    Ok(vec![TemplateDefinition::from_json(
        BASIC_README_TEMPLATE_JSON,
    )?])
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateVariable {
    pub name: String,
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateDefinition {
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    pub variables: Vec<TemplateVariable>,
    pub files: Vec<TemplateFile>,
}

impl TemplateDefinition {
    pub fn from_json(json: &str) -> Result<Self, TemplateLoadError> {
        let template: Self = serde_json::from_str(json).map_err(TemplateLoadError::InvalidJson)?;
        if template.schema_version != 1 {
            return Err(TemplateLoadError::UnsupportedSchemaVersion(
                template.schema_version,
            ));
        }
        if !valid_template_id(&template.id) {
            return Err(TemplateLoadError::InvalidField(
                "id must use 1-64 lowercase ASCII letters, digits, '-' or '_' and not be a reserved Windows name".to_string(),
            ));
        }
        if template.title.trim().is_empty() {
            return Err(TemplateLoadError::InvalidField(
                "title must not be empty".to_string(),
            ));
        }
        let mut variable_names = BTreeSet::new();
        for variable in &template.variables {
            if variable.name.trim().is_empty()
                || variable.name != variable.name.trim()
                || variable.name.contains("{{")
                || variable.name.contains("}}")
            {
                return Err(TemplateLoadError::InvalidField(
                    "variable names must be non-empty, trimmed, and contain no braces".to_string(),
                ));
            }
            if !variable_names.insert(variable.name.clone()) {
                return Err(TemplateLoadError::DuplicateVariable(variable.name.clone()));
            }
        }
        for file in &template.files {
            for source in [&file.path, &file.content] {
                for name in placeholder_names(source)? {
                    if !variable_names.contains(&name) {
                        return Err(TemplateLoadError::InvalidField(format!(
                            "template references undeclared variable: {name}"
                        )));
                    }
                }
            }
        }
        Ok(template)
    }
}

pub fn valid_template_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
        && !matches!(
            id.to_ascii_uppercase().as_str(),
            "CON"
                | "PRN"
                | "AUX"
                | "NUL"
                | "COM1"
                | "COM2"
                | "COM3"
                | "COM4"
                | "COM5"
                | "COM6"
                | "COM7"
                | "COM8"
                | "COM9"
                | "LPT1"
                | "LPT2"
                | "LPT3"
                | "LPT4"
                | "LPT5"
                | "LPT6"
                | "LPT7"
                | "LPT8"
                | "LPT9"
        )
}

#[derive(Debug, thiserror::Error)]
pub enum TemplateLoadError {
    #[error("invalid template JSON: {0}")]
    InvalidJson(serde_json::Error),
    #[error("unsupported template schema version: {0}")]
    UnsupportedSchemaVersion(u32),
    #[error("invalid template field: {0}")]
    InvalidField(String),
    #[error("duplicate template variable: {0}")]
    DuplicateVariable(String),
}

fn placeholder_names(source: &str) -> Result<Vec<String>, TemplateLoadError> {
    let mut names = Vec::new();
    let mut remainder = source;
    while let Some(start) = remainder.find("{{") {
        let variable_start = start + 2;
        let Some(relative_end) = remainder[variable_start..].find("}}") else {
            return Err(TemplateLoadError::InvalidField(
                "template variable is not closed".to_string(),
            ));
        };
        let end = variable_start + relative_end;
        let name = remainder[variable_start..end].trim();
        if name.is_empty() {
            return Err(TemplateLoadError::InvalidField(
                "template variable name must not be empty".to_string(),
            ));
        }
        names.push(name.to_string());
        remainder = &remainder[end + 2..];
    }
    Ok(names)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationRequest {
    pub destination: PathBuf,
    pub files: Vec<TemplateFile>,
    pub variables: BTreeMap<String, String>,
    pub overwrite: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAction {
    Create,
    Overwrite,
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedFile {
    pub relative_path: PathBuf,
    pub content: String,
    pub action: FileAction,
    pub target_revision: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationPlan {
    destination: PathBuf,
    files: Vec<PlannedFile>,
}

impl GenerationPlan {
    pub fn files(&self) -> &[PlannedFile] {
        &self.files
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileOutcome {
    Created,
    Overwritten,
    SkippedConflict,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedFile {
    pub relative_path: PathBuf,
    pub outcome: FileOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionReport {
    files: Vec<ExecutedFile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    Complete,
    Conflict,
    PartialFailure,
    Failed,
}

impl ExecutionReport {
    pub fn files(&self) -> &[ExecutedFile] {
        &self.files
    }

    pub fn status(&self) -> ExecutionStatus {
        let failed = self
            .files
            .iter()
            .filter(|file| matches!(file.outcome, FileOutcome::Failed(_)))
            .count();
        let conflicts = self
            .files
            .iter()
            .filter(|file| matches!(file.outcome, FileOutcome::SkippedConflict))
            .count();
        match failed {
            0 if conflicts > 0 => ExecutionStatus::Conflict,
            0 => ExecutionStatus::Complete,
            count if count == self.files.len() => ExecutionStatus::Failed,
            _ => ExecutionStatus::PartialFailure,
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum GenerationError {
    #[error("input provides an undeclared template variable: {0}")]
    UndeclaredVariable(String),
    #[error("template references missing variable: {0}")]
    MissingVariable(String),
    #[error("template variable is not closed")]
    UnclosedVariable,
    #[error("generated path is outside the destination: {0}")]
    UnsafePath(PathBuf),
    #[error("multiple template files render to the same path: {0}")]
    DuplicatePath(PathBuf),
    #[error("generated paths conflict as files and directories: {0}")]
    PathConflict(PathBuf),
    #[error("rendered template output exceeds 64 MiB")]
    RenderedOutputTooLarge,
    #[error("failed to inspect generated path {0}: {1}")]
    PathInspection(PathBuf, String),
}

impl GenerationError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::UndeclaredVariable(_) => "undeclared_variable",
            Self::MissingVariable(_) => "missing_variable",
            Self::UnclosedVariable => "unclosed_variable",
            Self::UnsafePath(_) => "unsafe_path",
            Self::DuplicatePath(_) => "duplicate_path",
            Self::PathConflict(_) => "path_conflict",
            Self::RenderedOutputTooLarge => "rendered_output_too_large",
            Self::PathInspection(_, _) => "path_inspection_failed",
        }
    }
}

#[derive(Debug, Default)]
pub struct FileGenerator;

static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

impl FileGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn plan_template(
        &self,
        template: &TemplateDefinition,
        destination: impl AsRef<Path>,
        mut variables: BTreeMap<String, String>,
        overwrite: bool,
    ) -> Result<GenerationPlan, GenerationError> {
        let declared_variables = template
            .variables
            .iter()
            .map(|variable| variable.name.as_str())
            .collect::<BTreeSet<_>>();
        if let Some(name) = variables
            .keys()
            .find(|name| !declared_variables.contains(name.as_str()))
        {
            return Err(GenerationError::UndeclaredVariable(name.clone()));
        }
        for variable in &template.variables {
            if let Some(default) = &variable.default {
                variables
                    .entry(variable.name.clone())
                    .or_insert_with(|| default.clone());
            }
            if !variable.required && !variables.contains_key(&variable.name) {
                variables.insert(variable.name.clone(), String::new());
            }
            if variable.required && !variables.contains_key(&variable.name) {
                return Err(GenerationError::MissingVariable(variable.name.clone()));
            }
        }
        self.plan(GenerationRequest {
            destination: destination.as_ref().to_path_buf(),
            files: template.files.clone(),
            variables,
            overwrite,
        })
    }

    pub fn plan(&self, request: GenerationRequest) -> Result<GenerationPlan, GenerationError> {
        let mut rendered_keys = BTreeSet::new();
        let mut rendered_bytes = 0usize;
        let mut files = Vec::with_capacity(request.files.len());
        for file in request.files {
            let rendered_path = render(
                &file.path,
                &request.variables,
                MAX_RENDERED_OUTPUT_BYTES.saturating_sub(rendered_bytes),
            )?;
            rendered_bytes = rendered_bytes
                .checked_add(rendered_path.len())
                .filter(|total| *total <= MAX_RENDERED_OUTPUT_BYTES)
                .ok_or(GenerationError::RenderedOutputTooLarge)?;
            let relative_path = PathBuf::from(rendered_path);
            validate_relative_path(&relative_path)?;
            let path_key = relative_path.to_string_lossy().to_lowercase();
            if rendered_keys.contains(&path_key) {
                return Err(GenerationError::DuplicatePath(relative_path));
            }
            let directory_prefix = format!("{path_key}/");
            if rendered_keys
                .range(directory_prefix.clone()..)
                .next()
                .is_some_and(|existing: &String| existing.starts_with(&directory_prefix))
            {
                return Err(GenerationError::PathConflict(relative_path));
            }
            for (index, _) in path_key.match_indices('/') {
                let parent = &path_key[..index];
                if rendered_keys.contains(parent) {
                    return Err(GenerationError::PathConflict(relative_path));
                }
            }
            rendered_keys.insert(path_key);
            let target = request.destination.join(&relative_path);
            let target_metadata = match std::fs::symlink_metadata(&target) {
                Ok(metadata) => Some(metadata),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => {
                    return Err(GenerationError::PathInspection(
                        relative_path,
                        error.to_string(),
                    ));
                }
            };
            let action = if let Some(metadata) = &target_metadata {
                if request.overwrite && metadata.is_file() {
                    FileAction::Overwrite
                } else {
                    FileAction::Conflict
                }
            } else {
                FileAction::Create
            };
            let content = render(
                &file.content,
                &request.variables,
                MAX_RENDERED_OUTPUT_BYTES.saturating_sub(rendered_bytes),
            )?;
            rendered_bytes = rendered_bytes
                .checked_add(content.len())
                .filter(|total| *total <= MAX_RENDERED_OUTPUT_BYTES)
                .ok_or(GenerationError::RenderedOutputTooLarge)?;
            files.push(PlannedFile {
                relative_path,
                content,
                action,
                target_revision: target_metadata.as_ref().map(metadata_revision),
            });
        }

        Ok(GenerationPlan {
            destination: request.destination,
            files,
        })
    }

    pub fn execute(&self, plan: &GenerationPlan) -> ExecutionReport {
        let destination = Dir::open_ambient_dir(&plan.destination, ambient_authority());
        let files = plan
            .files
            .iter()
            .map(|file| {
                if file.action == FileAction::Conflict {
                    return ExecutedFile {
                        relative_path: file.relative_path.clone(),
                        outcome: FileOutcome::SkippedConflict,
                    };
                }
                let destination = match destination.as_ref() {
                    Ok(destination) => destination,
                    Err(error) => {
                        return ExecutedFile {
                            relative_path: file.relative_path.clone(),
                            outcome: FileOutcome::Failed(error.to_string()),
                        };
                    }
                };
                let parent_result = file
                    .relative_path
                    .parent()
                    .map(|parent| destination.create_dir_all(parent))
                    .transpose();
                let outcome = match parent_result {
                    Err(error) => FileOutcome::Failed(error.to_string()),
                    Ok(_) => write_file_atomically(destination, file),
                };
                ExecutedFile {
                    relative_path: file.relative_path.clone(),
                    outcome,
                }
            })
            .collect();

        ExecutionReport { files }
    }
}

fn write_file_atomically(destination: &Dir, file: &PlannedFile) -> FileOutcome {
    let parent_path = file.relative_path.parent().unwrap_or_else(|| Path::new(""));
    let file_name = match file.relative_path.file_name() {
        Some(file_name) => file_name,
        None => return FileOutcome::Failed("generated path has no file name".to_string()),
    };
    let parent = match if parent_path.as_os_str().is_empty() {
        destination.try_clone()
    } else {
        destination.open_dir(parent_path)
    } {
        Ok(parent) => parent,
        Err(error) => return FileOutcome::Failed(error.to_string()),
    };
    let overwrite_permissions = if file.action == FileAction::Overwrite {
        match parent.symlink_metadata(file_name) {
            Ok(metadata)
                if file.target_revision.as_deref() == Some(&cap_metadata_revision(&metadata)) =>
            {
                Some(metadata.permissions())
            }
            Ok(_) => {
                return FileOutcome::Failed(
                    "planned overwrite target changed after preview".to_string(),
                );
            }
            Err(_) => {
                return FileOutcome::Failed(
                    "planned overwrite target no longer exists".to_string(),
                );
            }
        }
    } else {
        None
    };

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(windows)]
    {
        use cap_std::fs::OpenOptionsExt;
        // Renaming the completed file by handle requires DELETE in addition to write access.
        options.access_mode(0x4001_0000); // GENERIC_WRITE | DELETE
    }
    let (temporary_name, mut temporary) = match (0..32).find_map(|_| {
        let name = format!(
            ".bkmt-{}-{}.tmp",
            std::process::id(),
            TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed),
        );
        match parent.open_with(&name, &options) {
            Ok(file) => Some(Ok((name, file))),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => None,
            Err(error) => Some(Err(error)),
        }
    }) {
        Some(Ok(value)) => value,
        Some(Err(error)) => return FileOutcome::Failed(error.to_string()),
        None => return FileOutcome::Failed("could not allocate a temporary file".to_string()),
    };
    if let Some(permissions) = overwrite_permissions
        && let Err(error) = parent.set_permissions(&temporary_name, permissions)
    {
        let _ = parent.remove_file(&temporary_name);
        return FileOutcome::Failed(error.to_string());
    }
    let write_result = temporary
        .write_all(file.content.as_bytes())
        .and_then(|()| temporary.flush())
        .and_then(|()| temporary.sync_all());
    if let Err(error) = write_result {
        let _ = parent.remove_file(&temporary_name);
        return FileOutcome::Failed(error.to_string());
    }

    if file.action == FileAction::Overwrite {
        match parent.symlink_metadata(file_name) {
            Ok(metadata)
                if file.target_revision.as_deref() == Some(&cap_metadata_revision(&metadata)) => {}
            _ => {
                let _ = parent.remove_file(&temporary_name);
                return FileOutcome::Failed(
                    "planned overwrite target changed after preview".to_string(),
                );
            }
        }
    }
    let install_result = match file.action {
        FileAction::Create => install::create(&parent, &temporary, &temporary_name, file_name),
        FileAction::Overwrite => parent.rename(&temporary_name, &parent, file_name),
        FileAction::Conflict => unreachable!("handled before writing"),
    };
    if install_result.is_err() {
        let _ = parent.remove_file(&temporary_name);
    }
    let outcome = match install_result {
        Ok(()) if file.action == FileAction::Overwrite => FileOutcome::Overwritten,
        Ok(()) => FileOutcome::Created,
        Err(error)
            if file.action == FileAction::Create
                && error.kind() == std::io::ErrorKind::AlreadyExists =>
        {
            FileOutcome::SkippedConflict
        }
        Err(error) => FileOutcome::Failed(error.to_string()),
    };
    #[cfg(unix)]
    if matches!(outcome, FileOutcome::Created | FileOutcome::Overwritten) {
        let _ = parent.open(".").and_then(|directory| directory.sync_all());
    }
    outcome
}

fn metadata_revision(metadata: &std::fs::Metadata) -> String {
    let modified = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    let kind = if metadata.file_type().is_file() {
        "file"
    } else if metadata.file_type().is_dir() {
        "dir"
    } else if metadata.file_type().is_symlink() {
        "symlink"
    } else {
        "other"
    };
    format!("{kind}:{}:{modified}", metadata.len())
}

fn cap_metadata_revision(metadata: &cap_std::fs::Metadata) -> String {
    let modified = metadata
        .modified()
        .ok()
        .and_then(|value| value.into_std().duration_since(std::time::UNIX_EPOCH).ok())
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    let kind = if metadata.is_file() {
        "file"
    } else if metadata.is_dir() {
        "dir"
    } else if metadata.is_symlink() {
        "symlink"
    } else {
        "other"
    };
    format!("{kind}:{}:{modified}", metadata.len())
}

fn validate_relative_path(path: &Path) -> Result<(), GenerationError> {
    let text = path.to_string_lossy();
    // Inspect the raw spelling: Path::components normalizes away aliases such as './'.
    let unsafe_component = text.split('/').any(|component| {
        let stem = component
            .split('.')
            .next()
            .unwrap_or_default()
            .trim_end_matches(' ')
            .to_ascii_uppercase();
        let reserved = matches!(
            stem.as_str(),
            "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
        ) || ["COM", "LPT"].iter().any(|prefix| {
            stem.strip_prefix(prefix).is_some_and(|suffix| {
                matches!(
                    suffix,
                    "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                )
            })
        });
        component.is_empty()
            || component.ends_with(['.', ' '])
            || reserved
            || component.chars().any(|character| {
                character <= '\u{1f}'
                    || matches!(character, '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*')
            })
    });
    if unsafe_component {
        return Err(GenerationError::UnsafePath(path.to_path_buf()));
    }
    Ok(())
}

fn render(
    template: &str,
    variables: &BTreeMap<String, String>,
    limit: usize,
) -> Result<String, GenerationError> {
    let mut output = String::with_capacity(template.len());
    let mut remainder = template;

    while let Some(start) = remainder.find("{{") {
        let literal = &remainder[..start];
        if output.len().saturating_add(literal.len()) > limit {
            return Err(GenerationError::RenderedOutputTooLarge);
        }
        output.push_str(literal);
        let variable_start = start + 2;
        let Some(relative_end) = remainder[variable_start..].find("}}") else {
            return Err(GenerationError::UnclosedVariable);
        };
        let end = variable_start + relative_end;
        let name = remainder[variable_start..end].trim();
        let value = variables
            .get(name)
            .ok_or_else(|| GenerationError::MissingVariable(name.to_string()))?;
        if output.len().saturating_add(value.len()) > limit {
            return Err(GenerationError::RenderedOutputTooLarge);
        }
        output.push_str(value);
        remainder = &remainder[end + 2..];
    }
    if output.len().saturating_add(remainder.len()) > limit {
        return Err(GenerationError::RenderedOutputTooLarge);
    }
    output.push_str(remainder);
    Ok(output)
}
