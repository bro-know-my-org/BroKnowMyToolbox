use crate::DesktopState;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDiagnostics {
    pub data_root: String,
    pub data_root_source: String,
}

pub fn runtime_diagnostics(state: &DesktopState) -> RuntimeDiagnostics {
    RuntimeDiagnostics {
        data_root: state.data_root.to_string_lossy().into_owned(),
        data_root_source: match state.data_root_source {
            toolbox_core::DataRootSource::Explicit => "explicit",
            toolbox_core::DataRootSource::Environment => "environment",
            toolbox_core::DataRootSource::Portable => "portable",
            toolbox_core::DataRootSource::System => "system",
        }
        .to_string(),
    }
}

#[tauri::command]
pub fn runtime_diagnostics_command(state: tauri::State<'_, DesktopState>) -> RuntimeDiagnostics {
    runtime_diagnostics(&state)
}
