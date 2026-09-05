use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DataRootSource {
    Explicit,
    Environment,
    Portable,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataRootCandidates {
    pub explicit: Option<PathBuf>,
    pub environment: Option<PathBuf>,
    pub portable: Option<PathBuf>,
    pub system: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataRootDiscovery {
    pub explicit: Option<PathBuf>,
    pub environment: Option<PathBuf>,
    pub executable: PathBuf,
    pub portable_requested: bool,
    pub system: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDataRoot {
    pub path: PathBuf,
    pub source: DataRootSource,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DataRootError {
    #[error("the {0:?} data root candidate is empty")]
    EmptyPath(DataRootSource),
    #[error("the {0:?} data root candidate must be absolute")]
    RelativePath(DataRootSource),
}

pub fn resolve_data_root(
    candidates: DataRootCandidates,
) -> Result<ResolvedDataRoot, DataRootError> {
    let (path, source) = if let Some(path) = candidates.explicit {
        (path, DataRootSource::Explicit)
    } else if let Some(path) = candidates.environment {
        (path, DataRootSource::Environment)
    } else if let Some(path) = candidates.portable {
        (path, DataRootSource::Portable)
    } else {
        (candidates.system, DataRootSource::System)
    };

    if path.as_os_str().is_empty() {
        return Err(DataRootError::EmptyPath(source));
    }
    if !path.is_absolute() {
        return Err(DataRootError::RelativePath(source));
    }

    Ok(ResolvedDataRoot { path, source })
}

fn portable_package_root(executable: &Path) -> Option<&Path> {
    if !executable.is_absolute() {
        return None;
    }
    #[cfg(target_os = "macos")]
    if let Some(bundle) = executable.ancestors().find(|ancestor| {
        ancestor
            .extension()
            .is_some_and(|extension| extension == "app")
            && executable.starts_with(ancestor.join("Contents").join("MacOS"))
    }) {
        return bundle.parent();
    }
    executable.parent()
}

fn writable_portable_data_root(path: PathBuf) -> Option<PathBuf> {
    std::fs::create_dir_all(&path)
        .and_then(|()| tempfile::NamedTempFile::new_in(&path).map(|_| ()))
        .ok()
        .map(|()| path)
}

pub fn discover_data_root(discovery: DataRootDiscovery) -> Result<ResolvedDataRoot, DataRootError> {
    if let Some(path) = discovery.explicit {
        if !path.is_absolute() {
            return Err(DataRootError::RelativePath(DataRootSource::Explicit));
        }
        return resolve_data_root(DataRootCandidates {
            explicit: Some(path),
            environment: None,
            portable: None,
            system: discovery.system,
        });
    }
    if let Some(path) = discovery.environment {
        if !path.is_absolute() {
            return Err(DataRootError::RelativePath(DataRootSource::Environment));
        }
        return resolve_data_root(DataRootCandidates {
            explicit: None,
            environment: Some(path),
            portable: None,
            system: discovery.system,
        });
    }
    let portable = portable_package_root(&discovery.executable).and_then(|root| {
        (discovery.portable_requested || root.join("portable.bkmt").is_file())
            .then(|| root.join("data"))
            .and_then(writable_portable_data_root)
    });

    resolve_data_root(DataRootCandidates {
        explicit: None,
        environment: None,
        portable,
        system: discovery.system,
    })
}
