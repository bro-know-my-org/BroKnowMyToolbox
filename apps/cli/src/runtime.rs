use std::path::PathBuf;

use crate::error::CliError;

pub(crate) fn resolve_data_root(
    explicit: Option<PathBuf>,
    portable: bool,
    json: bool,
) -> Result<toolbox_core::ResolvedDataRoot, CliError> {
    let environment = std::env::var_os("BKMT_DATA_DIR").map(PathBuf::from);
    if explicit.is_some() || environment.is_some() {
        return toolbox_core::discover_data_root(toolbox_core::DataRootDiscovery {
            explicit,
            environment,
            executable: PathBuf::new(),
            portable_requested: false,
            system: PathBuf::new(),
        })
        .map_err(|error| match error {
            toolbox_core::DataRootError::EmptyPath(toolbox_core::DataRootSource::Explicit)
            | toolbox_core::DataRootError::EmptyPath(toolbox_core::DataRootSource::Environment)
            | toolbox_core::DataRootError::RelativePath(toolbox_core::DataRootSource::Explicit)
            | toolbox_core::DataRootError::RelativePath(
                toolbox_core::DataRootSource::Environment,
            ) => CliError::input("invalid_data_root", error.to_string(), json),
            _ => CliError::operation("data_root_unavailable", error.to_string(), 1, json),
        });
    }
    let executable = std::env::current_exe().map_err(|error| {
        CliError::operation(
            "data_root_unavailable",
            format!("failed to locate the bkmt executable: {error}"),
            1,
            json,
        )
    })?;
    let system =
        directories::ProjectDirs::from("io.github", "bro-know-my-org", "Bro Know My Toolbox")
            .ok_or_else(|| {
                CliError::operation(
                    "data_root_unavailable",
                    "failed to determine the system data directory",
                    1,
                    json,
                )
            })?
            .data_local_dir()
            .to_path_buf();

    toolbox_core::discover_data_root(toolbox_core::DataRootDiscovery {
        explicit: None,
        environment: None,
        executable,
        portable_requested: portable
            || std::env::var("BKMT_PORTABLE")
                .is_ok_and(|value| matches!(value.as_str(), "1" | "true" | "yes")),
        system,
    })
    .map_err(|error| CliError::operation("data_root_unavailable", error.to_string(), 1, json))
}

pub(crate) fn parse_capability(value: &str) -> Result<toolbox_core::CapabilityId, String> {
    match value {
        "credentials:ai" => Ok(toolbox_core::CapabilityId::CredentialsAi),
        "filesystem:read" => Ok(toolbox_core::CapabilityId::FilesystemRead),
        "filesystem:write" => Ok(toolbox_core::CapabilityId::FilesystemWrite),
        "network:spark" => Ok(toolbox_core::CapabilityId::NetworkSpark),
        _ => Err(format!("unknown capability: {value}")),
    }
}

pub(crate) fn check_tool_capability(
    data_root: &toolbox_core::ResolvedDataRoot,
    tool_id: &str,
    capability: toolbox_core::CapabilityId,
    capability_name: &'static str,
    json: bool,
) -> Result<(), CliError> {
    let authorizer = toolbox_core::CapabilityAuthorizer::new(toolbox_core::FileConsentStore::new(
        &data_root.path,
    ))
    .map_err(|error| CliError::operation("authorization_failed", error.to_string(), 1, json))?;
    authorizer.check(tool_id, capability).map_err(|error| {
        let code = if matches!(error, toolbox_core::AuthorizationError::ConsentDenied) {
            "consent_denied"
        } else if matches!(error, toolbox_core::AuthorizationError::ConsentRequired) {
            "consent_required"
        } else {
            return CliError::operation("authorization_failed", error.to_string(), 1, json);
        };
        CliError::capability(code, error.to_string(), tool_id, capability_name, json)
    })
}
