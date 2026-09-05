//! Shared product identity and contracts used by every application adapter.

mod authorization;
mod config;
mod data_root;
mod tools;

pub use authorization::{
    AuthorizationError, CapabilityAuthorizer, ConsentDecision, ConsentStore, FileConsentStore,
    MemoryConsentStore,
};
pub use config::{AppConfig, ConfigError, DensityMode, FileConfigStore, ThemeMode};
pub use data_root::{
    DataRootCandidates, DataRootDiscovery, DataRootError, DataRootSource, ResolvedDataRoot,
    discover_data_root, resolve_data_root,
};
pub use tools::{CapabilityId, ToolCatalogError, ToolDescriptor, embedded_tool_catalog};

/// Product version shared by the desktop and CLI adapters.
pub const WORKSPACE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Stable application identifier used by desktop packaging and data paths.
pub const APPLICATION_IDENTIFIER: &str = "io.github.bro-know-my-org.bro-know-my-toolbox";
