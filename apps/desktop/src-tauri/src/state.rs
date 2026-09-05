use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DesktopState {
    pub(crate) data_root: PathBuf,
    pub(crate) data_root_source: toolbox_core::DataRootSource,
}

impl DesktopState {
    pub fn new(data_root: PathBuf) -> Result<Self, toolbox_core::DataRootError> {
        let resolved = toolbox_core::resolve_data_root(toolbox_core::DataRootCandidates {
            explicit: Some(data_root),
            environment: None,
            portable: None,
            system: PathBuf::new(),
        })?;
        Ok(Self {
            data_root: resolved.path,
            data_root_source: resolved.source,
        })
    }

    pub fn discover() -> Result<Self, String> {
        if let Some(environment) = std::env::var_os("BKMT_DATA_DIR").map(PathBuf::from) {
            let resolved = toolbox_core::discover_data_root(toolbox_core::DataRootDiscovery {
                explicit: None,
                environment: Some(environment),
                executable: PathBuf::new(),
                portable_requested: false,
                system: PathBuf::new(),
            })
            .map_err(|error| error.to_string())?;
            return Ok(Self {
                data_root: resolved.path,
                data_root_source: resolved.source,
            });
        }
        let executable = std::env::current_exe()
            .map_err(|error| format!("failed to locate the desktop executable: {error}"))?;
        let system =
            directories::ProjectDirs::from("io.github", "bro-know-my-org", "Bro Know My Toolbox")
                .ok_or_else(|| "failed to determine the system data directory".to_string())?
                .data_local_dir()
                .to_path_buf();
        let resolved = toolbox_core::discover_data_root(toolbox_core::DataRootDiscovery {
            explicit: None,
            environment: None,
            executable,
            portable_requested: std::env::var("BKMT_PORTABLE")
                .is_ok_and(|value| matches!(value.as_str(), "1" | "true" | "yes")),
            system,
        })
        .map_err(|error| error.to_string())?;
        Ok(Self {
            data_root: resolved.path,
            data_root_source: resolved.source,
        })
    }
}
