use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SparkHostAuthorizer {
    data_root: PathBuf,
}

impl SparkHostAuthorizer {
    pub fn new(data_root: impl AsRef<Path>) -> Self {
        Self {
            data_root: data_root.as_ref().to_path_buf(),
        }
    }
}

impl bkmsa_tauri::HostAuthorizer for SparkHostAuthorizer {
    fn authorize(&self, capability: bkmsa_tauri::HostCapability) -> Result<(), String> {
        let (capability, capability_name) = toolbox_capability(capability);
        let authorizer = toolbox_core::CapabilityAuthorizer::new(
            toolbox_core::FileConsentStore::new(&self.data_root),
        )
        .map_err(|error| authorization_error("authorization_failed", capability_name, error))?;
        authorizer
            .check("spark-analyzer", capability)
            .map_err(|error| {
                let code = match error {
                    toolbox_core::AuthorizationError::ConsentRequired => "consent_required",
                    toolbox_core::AuthorizationError::ConsentDenied => "consent_denied",
                    _ => "authorization_failed",
                };
                authorization_error(code, capability_name, error)
            })
    }
}

fn toolbox_capability(
    capability: bkmsa_tauri::HostCapability,
) -> (toolbox_core::CapabilityId, &'static str) {
    match capability {
        bkmsa_tauri::HostCapability::Network => {
            (toolbox_core::CapabilityId::NetworkSpark, "network:spark")
        }
        bkmsa_tauri::HostCapability::Credentials => {
            (toolbox_core::CapabilityId::CredentialsAi, "credentials:ai")
        }
        bkmsa_tauri::HostCapability::FilesystemWrite => (
            toolbox_core::CapabilityId::FilesystemWrite,
            "filesystem:write",
        ),
    }
}

pub fn set_spark_consent(
    state: &crate::DesktopState,
    capability: &str,
    allowed: bool,
) -> Result<(), String> {
    let capability = match capability {
        "credentials:ai" => toolbox_core::CapabilityId::CredentialsAi,
        "filesystem:write" => toolbox_core::CapabilityId::FilesystemWrite,
        "network:spark" => toolbox_core::CapabilityId::NetworkSpark,
        _ => return Err(format!("unknown Spark capability: {capability}")),
    };
    let decision = if allowed {
        toolbox_core::ConsentDecision::Allowed
    } else {
        toolbox_core::ConsentDecision::Denied
    };
    let mut authorizer = toolbox_core::CapabilityAuthorizer::new(
        toolbox_core::FileConsentStore::new(&state.data_root),
    )
    .map_err(|error| error.to_string())?;
    authorizer
        .record("spark-analyzer", capability, decision)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn set_spark_consent_command(
    capability: String,
    allowed: bool,
    state: tauri::State<'_, crate::DesktopState>,
) -> Result<(), String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || set_spark_consent(&state, &capability, allowed))
        .await
        .map_err(|error| error.to_string())?
}

fn authorization_error(code: &str, capability: &str, error: impl std::fmt::Display) -> String {
    serde_json::json!({
        "code": code,
        "capability": capability,
        "message": error.to_string(),
    })
    .to_string()
}
