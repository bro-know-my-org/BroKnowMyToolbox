use std::io::Read;
use std::path::Path;

use crate::{TemplateDefinition, TemplateLoadError, built_in_templates};

pub const MAX_USER_TEMPLATE_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateSource {
    BuiltIn,
    User,
}

impl TemplateSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BuiltIn => "built_in",
            Self::User => "user",
        }
    }
}

#[derive(Debug)]
pub struct TemplateEntry {
    pub definition: TemplateDefinition,
    pub source: TemplateSource,
    pub json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateWarning {
    pub file_name: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug)]
pub struct TemplateCatalog {
    pub templates: Vec<TemplateEntry>,
    pub warnings: Vec<TemplateWarning>,
}

#[derive(Debug, thiserror::Error)]
pub enum TemplateCatalogError {
    #[error("failed to read template storage: {0}")]
    Unreadable(#[from] std::io::Error),
    #[error("user template exceeds 1 MiB")]
    TooLarge,
    #[error("{0}")]
    InvalidTemplate(#[from] TemplateLoadError),
    #[error("template is not valid UTF-8: {0}")]
    InvalidEncoding(#[from] std::string::FromUtf8Error),
    #[error("{0}")]
    InvalidId(String),
    #[error("failed to serialize built-in template: {0}")]
    Serialization(serde_json::Error),
}

impl TemplateCatalogError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Unreadable(_) => "template_unreadable",
            Self::TooLarge => "user_template_too_large",
            Self::InvalidTemplate(_) | Self::InvalidEncoding(_) | Self::Serialization(_) => {
                "invalid_template"
            }
            Self::InvalidId(_) => "invalid_template_id",
        }
    }
}

/// Read a bounded regular file from one no-follow handle; never wait on a FIFO.
pub fn read_template_file(path: &Path) -> Result<String, TemplateCatalogError> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x0020_0000); // FILE_FLAG_OPEN_REPARSE_POINT
    }
    let file = options.open(path)?;
    let metadata = file.metadata()?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "template must not be a reparse point",
            )
            .into());
        }
    }
    if !metadata.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "template must be a regular file",
        )
        .into());
    }
    if metadata.len() > MAX_USER_TEMPLATE_BYTES {
        return Err(TemplateCatalogError::TooLarge);
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_USER_TEMPLATE_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_USER_TEMPLATE_BYTES {
        return Err(TemplateCatalogError::TooLarge);
    }
    Ok(String::from_utf8(bytes)?)
}

/// Keep valid templates discoverable when individual user files are damaged.
/// Failure to enumerate the directory itself remains an error, never an empty catalog.
pub fn load_template_catalog(directory: &Path) -> Result<TemplateCatalog, TemplateCatalogError> {
    let mut catalog = TemplateCatalog {
        templates: Vec::new(),
        warnings: Vec::new(),
    };
    for definition in built_in_templates()? {
        let json =
            serde_json::to_string(&definition).map_err(TemplateCatalogError::Serialization)?;
        catalog.templates.push(TemplateEntry {
            definition,
            source: TemplateSource::BuiltIn,
            json,
        });
    }
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if genuinely_missing_directory(directory)? {
                return Ok(catalog);
            }
            return Err(error.into());
        }
        Err(error) => return Err(error.into()),
    };
    let mut users = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let loaded = (|| {
            let json = read_template_file(&path)?;
            let definition = TemplateDefinition::from_json(&json)?;
            if path.file_stem().and_then(|value| value.to_str()) != Some(&definition.id) {
                return Err(TemplateCatalogError::InvalidId(format!(
                    "user template filename must match id: {}",
                    definition.id
                )));
            }
            if catalog
                .templates
                .iter()
                .any(|built_in| built_in.definition.id == definition.id)
            {
                return Err(TemplateCatalogError::InvalidId(format!(
                    "user template id is reserved by a built-in template: {}",
                    definition.id
                )));
            }
            Ok(TemplateEntry {
                definition,
                source: TemplateSource::User,
                json,
            })
        })();
        match loaded {
            Ok(template) => users.push(template),
            Err(error) => catalog.warnings.push(TemplateWarning {
                file_name: entry.file_name().to_string_lossy().into_owned(),
                code: error.code().to_owned(),
                message: error.to_string(),
            }),
        }
    }
    users.sort_by(|left, right| left.definition.id.cmp(&right.definition.id));
    catalog.templates.extend(users);
    catalog
        .warnings
        .sort_by(|left, right| left.file_name.cmp(&right.file_name));
    Ok(catalog)
}

fn genuinely_missing_directory(directory: &Path) -> std::io::Result<bool> {
    // A missing leaf beneath a dangling link or regular file is broken storage,
    // not an empty profile. Inspect the nearest existing ancestor without following it.
    for (depth, candidate) in directory.ancestors().enumerate() {
        let candidate = if candidate.as_os_str().is_empty() {
            Path::new(".")
        } else {
            candidate
        };
        match std::fs::symlink_metadata(candidate) {
            Ok(_) if depth == 0 => return Ok(false),
            Ok(_) => return std::fs::metadata(candidate).map(|metadata| metadata.is_dir()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        }
    }
    Ok(false)
}
