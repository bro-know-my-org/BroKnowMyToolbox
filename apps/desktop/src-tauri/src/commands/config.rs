use crate::{CommandError, DesktopState};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigDto {
    schema_version: u32,
    locale: String,
    theme: toolbox_core::ThemeMode,
    accent_color: String,
    font_scale_percent: u16,
    density: toolbox_core::DensityMode,
    reduced_motion: bool,
    favorite_tool_ids: Vec<String>,
    recent_tool_ids: Vec<String>,
}

impl From<toolbox_core::AppConfig> for AppConfigDto {
    fn from(config: toolbox_core::AppConfig) -> Self {
        Self {
            schema_version: config.schema_version,
            locale: config.locale,
            theme: config.theme,
            accent_color: config.accent_color,
            font_scale_percent: config.font_scale_percent,
            density: config.density,
            reduced_motion: config.reduced_motion,
            favorite_tool_ids: config.favorite_tool_ids,
            recent_tool_ids: config.recent_tool_ids,
        }
    }
}

impl From<AppConfigDto> for toolbox_core::AppConfig {
    fn from(config: AppConfigDto) -> Self {
        Self {
            schema_version: config.schema_version,
            locale: config.locale,
            theme: config.theme,
            accent_color: config.accent_color,
            font_scale_percent: config.font_scale_percent,
            density: config.density,
            reduced_motion: config.reduced_motion,
            favorite_tool_ids: config.favorite_tool_ids,
            recent_tool_ids: config.recent_tool_ids,
        }
    }
}

pub fn load_app_config(state: &DesktopState) -> Result<toolbox_core::AppConfig, CommandError> {
    toolbox_core::FileConfigStore::new(&state.data_root)
        .load()
        .map_err(|error| CommandError {
            code: error.code().to_string(),
            message: error.to_string(),
        })
}

pub fn save_app_config(
    state: &DesktopState,
    config: toolbox_core::AppConfig,
) -> Result<(), CommandError> {
    toolbox_core::FileConfigStore::new(&state.data_root)
        .save(&config)
        .map_err(|error| CommandError {
            code: error.code().to_string(),
            message: error.to_string(),
        })
}

#[tauri::command]
pub async fn load_app_config_command(
    state: tauri::State<'_, DesktopState>,
) -> Result<AppConfigDto, CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || load_app_config(&state).map(AppConfigDto::from))
        .await
        .map_err(|error| CommandError {
            code: "config_task_failed".to_string(),
            message: error.to_string(),
        })?
}

#[tauri::command]
pub async fn save_app_config_command(
    state: tauri::State<'_, DesktopState>,
    config: AppConfigDto,
) -> Result<(), CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || save_app_config(&state, config.into()))
        .await
        .map_err(|error| CommandError {
            code: "config_task_failed".to_string(),
            message: error.to_string(),
        })?
}

#[cfg(test)]
mod tests {
    use super::AppConfigDto;

    #[test]
    fn desktop_config_ipc_uses_camel_case_without_changing_the_toml_model() {
        let value = serde_json::to_value(AppConfigDto::from(toolbox_core::AppConfig::default()))
            .expect("desktop config should serialize");

        assert_eq!(value["schemaVersion"], 1);
        assert_eq!(value["accentColor"], "#78a9ff");
        assert!(value.get("schema_version").is_none());
        assert!(value.get("favoriteToolIds").is_some());
        assert!(value.get("recentToolIds").is_some());

        let decoded: AppConfigDto =
            serde_json::from_value(value).expect("desktop config should deserialize");
        let core = toolbox_core::AppConfig::from(decoded);
        assert_eq!(core.schema_version, 1);
        assert_eq!(core.accent_color, "#78a9ff");
    }
}
