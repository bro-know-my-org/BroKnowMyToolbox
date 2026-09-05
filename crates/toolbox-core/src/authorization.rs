use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::{CapabilityId, embedded_tool_catalog};

const CONSENT_LOCK_TIMEOUT: Duration = Duration::from_millis(500);
const LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(10);
const MAX_CONSENT_FILE_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentDecision {
    Allowed,
    Denied,
}

pub trait ConsentStore {
    fn get(
        &self,
        tool_id: &str,
        capability: CapabilityId,
    ) -> Result<Option<ConsentDecision>, String>;

    fn set(
        &mut self,
        tool_id: &str,
        capability: CapabilityId,
        decision: ConsentDecision,
    ) -> Result<(), String>;
}

#[derive(Debug, Default)]
pub struct MemoryConsentStore {
    decisions: HashMap<(String, CapabilityId), ConsentDecision>,
}

impl ConsentStore for MemoryConsentStore {
    fn get(
        &self,
        tool_id: &str,
        capability: CapabilityId,
    ) -> Result<Option<ConsentDecision>, String> {
        Ok(self
            .decisions
            .get(&(tool_id.to_string(), capability))
            .copied())
    }

    fn set(
        &mut self,
        tool_id: &str,
        capability: CapabilityId,
        decision: ConsentDecision,
    ) -> Result<(), String> {
        self.decisions
            .insert((tool_id.to_string(), capability), decision);
        Ok(())
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct ConsentFile {
    schema_version: u32,
    decisions: BTreeMap<String, BTreeMap<CapabilityId, ConsentDecision>>,
}

impl Default for ConsentFile {
    fn default() -> Self {
        Self {
            schema_version: 1,
            decisions: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileConsentStore {
    data_path: PathBuf,
    lock_path: PathBuf,
}

impl FileConsentStore {
    pub fn new(data_root: impl AsRef<Path>) -> Self {
        let config = data_root.as_ref().join("config");
        Self {
            data_path: config.join("consents.json"),
            lock_path: config.join("consents.lock"),
        }
    }

    fn acquire_lock(&self, exclusive: bool) -> Result<File, String> {
        let parent = self
            .lock_path
            .parent()
            .ok_or_else(|| "consent lock path has no parent".to_string())?;
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create consent directory: {error}"))?;
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&self.lock_path)
            .map_err(|error| format!("failed to open consent lock: {error}"))?;
        let deadline = Instant::now() + CONSENT_LOCK_TIMEOUT;
        loop {
            let result = if exclusive {
                fs2::FileExt::try_lock_exclusive(&file)
            } else {
                fs2::FileExt::try_lock_shared(&file)
            };
            match result {
                Ok(()) => break,
                Err(error)
                    if error.kind() == std::io::ErrorKind::WouldBlock
                        && Instant::now() < deadline =>
                {
                    std::thread::sleep(LOCK_RETRY_INTERVAL);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    return Err("timed out waiting for consent lock".to_string());
                }
                Err(error) => return Err(format!("failed to lock consent storage: {error}")),
            }
        }
        Ok(file)
    }

    fn load(&self) -> Result<ConsentFile, String> {
        let initial = match std::fs::symlink_metadata(&self.data_path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(ConsentFile::default());
            }
            Err(error) => return Err(format!("failed to inspect consent storage: {error}")),
        };
        if !initial.file_type().is_file() {
            return Err("consent storage must be a regular file".to_string());
        }
        let mut file = File::open(&self.data_path)
            .map_err(|error| format!("failed to read consent storage: {error}"))?;
        let metadata = file
            .metadata()
            .map_err(|error| format!("failed to inspect consent storage: {error}"))?;
        if !metadata.is_file() || metadata.len() > MAX_CONSENT_FILE_BYTES {
            return Err("consent storage exceeds 64 KiB or is not a regular file".to_string());
        }
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        std::io::Read::by_ref(&mut file)
            .take(MAX_CONSENT_FILE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| format!("failed to read consent storage: {error}"))?;
        if bytes.len() as u64 > MAX_CONSENT_FILE_BYTES {
            return Err("consent storage exceeds 64 KiB".to_string());
        }
        let consents: ConsentFile = serde_json::from_slice(&bytes)
            .map_err(|error| format!("failed to parse consent storage: {error}"))?;
        if consents.schema_version != 1 {
            return Err(format!(
                "unsupported consent schema version: {}",
                consents.schema_version
            ));
        }
        Ok(consents)
    }

    fn save(&self, consents: &ConsentFile) -> Result<(), String> {
        let parent = self
            .data_path
            .parent()
            .ok_or_else(|| "consent data path has no parent".to_string())?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)
            .map_err(|error| format!("failed to create temporary consent file: {error}"))?;
        let serialized = serde_json::to_vec_pretty(consents)
            .map_err(|error| format!("failed to serialize consent storage: {error}"))?;
        if serialized.len() as u64 + 1 > MAX_CONSENT_FILE_BYTES {
            return Err("consent storage exceeds 64 KiB".to_string());
        }
        temporary
            .write_all(&serialized)
            .map_err(|error| format!("failed to write consent storage: {error}"))?;
        temporary
            .write_all(b"\n")
            .and_then(|()| temporary.flush())
            .and_then(|()| temporary.as_file().sync_all())
            .map_err(|error| format!("failed to flush consent storage: {error}"))?;
        temporary
            .persist(&self.data_path)
            .map_err(|error| format!("failed to replace consent storage: {}", error.error))?;

        #[cfg(unix)]
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| format!("failed to flush consent directory: {error}"))?;
        Ok(())
    }
}

impl ConsentStore for FileConsentStore {
    fn get(
        &self,
        tool_id: &str,
        capability: CapabilityId,
    ) -> Result<Option<ConsentDecision>, String> {
        let lock = self.acquire_lock(false)?;
        let result = self.load().map(|consents| {
            consents
                .decisions
                .get(tool_id)
                .and_then(|capabilities| capabilities.get(&capability))
                .copied()
        });
        let unlock = fs2::FileExt::unlock(&lock)
            .map_err(|error| format!("failed to unlock consent storage: {error}"));
        result.and_then(|value| unlock.map(|()| value))
    }

    fn set(
        &mut self,
        tool_id: &str,
        capability: CapabilityId,
        decision: ConsentDecision,
    ) -> Result<(), String> {
        let lock = self.acquire_lock(true)?;
        let result = self.load().and_then(|mut consents| {
            consents
                .decisions
                .entry(tool_id.to_string())
                .or_default()
                .insert(capability, decision);
            self.save(&consents)
        });
        let unlock = fs2::FileExt::unlock(&lock)
            .map_err(|error| format!("failed to unlock consent storage: {error}"));
        result.and(unlock)
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AuthorizationError {
    #[error("failed to load the built-in tool catalog: {0}")]
    InvalidCatalog(String),
    #[error("unknown tool: {0}")]
    UnknownTool(String),
    #[error("tool {tool_id} did not declare capability {capability:?}")]
    CapabilityNotDeclared {
        tool_id: String,
        capability: CapabilityId,
    },
    #[error("user consent is required")]
    ConsentRequired,
    #[error("user denied this capability")]
    ConsentDenied,
    #[error("consent storage failed: {0}")]
    Storage(String),
}

pub struct CapabilityAuthorizer<S> {
    store: S,
    declarations: HashMap<String, HashSet<CapabilityId>>,
}

impl<S: ConsentStore> CapabilityAuthorizer<S> {
    pub fn new(store: S) -> Result<Self, AuthorizationError> {
        let declarations = embedded_tool_catalog()
            .map_err(|error| AuthorizationError::InvalidCatalog(error.to_string()))?
            .into_iter()
            .map(|tool| (tool.id, tool.capabilities.into_iter().collect()))
            .collect();
        Ok(Self {
            store,
            declarations,
        })
    }

    fn ensure_declared(
        &self,
        tool_id: &str,
        capability: CapabilityId,
    ) -> Result<(), AuthorizationError> {
        let Some(capabilities) = self.declarations.get(tool_id) else {
            return Err(AuthorizationError::UnknownTool(tool_id.to_string()));
        };
        if !capabilities.contains(&capability) {
            return Err(AuthorizationError::CapabilityNotDeclared {
                tool_id: tool_id.to_string(),
                capability,
            });
        }
        Ok(())
    }

    pub fn check(&self, tool_id: &str, capability: CapabilityId) -> Result<(), AuthorizationError> {
        self.ensure_declared(tool_id, capability)?;
        match self
            .store
            .get(tool_id, capability)
            .map_err(AuthorizationError::Storage)?
        {
            Some(ConsentDecision::Allowed) => Ok(()),
            Some(ConsentDecision::Denied) => Err(AuthorizationError::ConsentDenied),
            None => Err(AuthorizationError::ConsentRequired),
        }
    }

    pub fn record(
        &mut self,
        tool_id: &str,
        capability: CapabilityId,
        decision: ConsentDecision,
    ) -> Result<(), AuthorizationError> {
        self.ensure_declared(tool_id, capability)?;
        self.store
            .set(tool_id, capability, decision)
            .map_err(AuthorizationError::Storage)
    }
}
