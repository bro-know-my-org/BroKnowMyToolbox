use std::{fs, path::PathBuf};

#[tauri::command]
pub fn create_file(path: String, content: String) -> Result<(), String> {
    let trimmed_path = path.trim();

    if trimmed_path.is_empty() {
        return Err("path cannot be empty".to_string());
    }

    let target_path = PathBuf::from(trimmed_path);

    if target_path.is_dir() {
        return Err("target path is a directory".to_string());
    }

    if let Some(parent) = target_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create parent directory: {error}"))?;
        }
    }

    fs::write(&target_path, content).map_err(|error| format!("failed to write file: {error}"))?;

    Ok(())
}
