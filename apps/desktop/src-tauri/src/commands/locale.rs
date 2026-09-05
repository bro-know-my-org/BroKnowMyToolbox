use std::collections::BTreeMap;
use std::io::Read;

use crate::{CommandError, DesktopState};

const MAX_LOCALE_PACK_BYTES: u64 = 1024 * 1024;

pub fn load_locale_override(
    state: &DesktopState,
    locale: &str,
) -> Result<BTreeMap<String, String>, CommandError> {
    if locale.is_empty()
        || !locale
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(CommandError {
            code: "invalid_locale".to_string(),
            message: "locale must contain only letters, numbers, '-' or '_'".to_string(),
        });
    }
    let path = state
        .data_root
        .join("locales")
        .join(format!("{locale}.json"));
    let file = match std::fs::File::open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(BTreeMap::new());
        }
        Err(error) => {
            return Err(CommandError {
                code: "locale_storage_failed".to_string(),
                message: error.to_string(),
            });
        }
    };
    let metadata = file.metadata().map_err(|error| CommandError {
        code: "locale_storage_failed".to_string(),
        message: error.to_string(),
    })?;
    if !metadata.is_file() {
        return Err(CommandError {
            code: "locale_storage_failed".to_string(),
            message: "locale pack must be a regular file".to_string(),
        });
    }
    if metadata.len() > MAX_LOCALE_PACK_BYTES {
        return Err(CommandError {
            code: "locale_pack_too_large".to_string(),
            message: "locale pack exceeds 1 MiB".to_string(),
        });
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_LOCALE_PACK_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| CommandError {
            code: "locale_storage_failed".to_string(),
            message: error.to_string(),
        })?;
    if bytes.len() as u64 > MAX_LOCALE_PACK_BYTES {
        return Err(CommandError {
            code: "locale_pack_too_large".to_string(),
            message: "locale pack exceeds 1 MiB".to_string(),
        });
    }
    let source = String::from_utf8(bytes).map_err(|error| CommandError {
        code: "invalid_locale_pack".to_string(),
        message: error.to_string(),
    })?;
    serde_json::from_str(&source).map_err(|error| CommandError {
        code: "invalid_locale_pack".to_string(),
        message: error.to_string(),
    })
}

#[tauri::command]
pub async fn load_locale_override_command(
    state: tauri::State<'_, DesktopState>,
    locale: String,
) -> Result<BTreeMap<String, String>, CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || load_locale_override(&state, &locale))
        .await
        .map_err(|error| CommandError {
            code: "locale_task_failed".to_string(),
            message: error.to_string(),
        })?
}
