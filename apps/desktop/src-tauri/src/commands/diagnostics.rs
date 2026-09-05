use crate::{CommandError, DesktopState};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDiagnostics {
    pub data_root: String,
    pub data_root_source: toolbox_core::DataRootSource,
}

pub fn runtime_diagnostics(state: &DesktopState) -> Result<RuntimeDiagnostics, CommandError> {
    let path = state.data_root.to_str().ok_or_else(|| CommandError {
        code: "data_root_not_unicode".to_string(),
        message: "the data directory cannot be represented as UTF-8".to_string(),
    })?;
    Ok(RuntimeDiagnostics {
        data_root: path.to_owned(),
        data_root_source: state.data_root_source,
    })
}

#[tauri::command]
pub fn runtime_diagnostics_command(
    state: tauri::State<'_, DesktopState>,
) -> Result<RuntimeDiagnostics, CommandError> {
    runtime_diagnostics(&state)
}
