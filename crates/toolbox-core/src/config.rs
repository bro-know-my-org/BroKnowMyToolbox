use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const CONFIG_LOCK_TIMEOUT: Duration = Duration::from_millis(500);
const LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(10);
const MAX_CONFIG_BYTES: u64 = 1024 * 1024;

fn open_config(path: &Path) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    options.open(path)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DensityMode {
    Comfortable,
    Compact,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct AppConfig {
    pub schema_version: u32,
    pub locale: String,
    pub theme: ThemeMode,
    pub accent_color: String,
    pub font_scale_percent: u16,
    pub density: DensityMode,
    pub reduced_motion: bool,
    pub favorite_tool_ids: Vec<String>,
    pub recent_tool_ids: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            locale: "system".to_string(),
            theme: ThemeMode::System,
            accent_color: "#78a9ff".to_string(),
            font_scale_percent: 100,
            density: DensityMode::Comfortable,
            reduced_motion: false,
            favorite_tool_ids: Vec::new(),
            recent_tool_ids: Vec::new(),
        }
    }
}

impl AppConfig {
    fn normalize(&mut self) -> Result<(), ConfigError> {
        let known_tools = crate::embedded_tool_catalog()
            .map_err(|error| ConfigError::InvalidValue(error.to_string()))?
            .into_iter()
            .map(|tool| tool.id)
            .collect::<HashSet<_>>();
        let mut favorites = HashSet::new();
        self.favorite_tool_ids
            .retain(|id| known_tools.contains(id.as_str()) && favorites.insert(id.clone()));
        let mut recents = HashSet::new();
        self.recent_tool_ids
            .retain(|id| known_tools.contains(id.as_str()) && recents.insert(id.clone()));
        self.recent_tool_ids.truncate(10);
        Ok(())
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if !matches!(self.locale.as_str(), "system" | "zh-CN" | "en-US") {
            return Err(ConfigError::InvalidValue(
                "locale must be system, zh-CN, or en-US".to_string(),
            ));
        }
        if !(85..=130).contains(&self.font_scale_percent) {
            return Err(ConfigError::InvalidValue(
                "font_scale_percent must be between 85 and 130".to_string(),
            ));
        }
        if self.accent_color.len() != 7
            || !self.accent_color.starts_with('#')
            || !self.accent_color[1..]
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(ConfigError::InvalidValue(
                "accent_color must use #RRGGBB format".to_string(),
            ));
        }
        if self.recent_tool_ids.len() > 10 {
            return Err(ConfigError::InvalidValue(
                "recent_tool_ids cannot contain more than 10 entries".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config storage failed: {0}")]
    Storage(String),
    #[error("unsupported config schema version: {0}")]
    UnsupportedSchemaVersion(u32),
    #[error("invalid config TOML: {0}")]
    InvalidToml(#[from] toml::de::Error),
    #[error("invalid config value: {0}")]
    InvalidValue(String),
}

impl ConfigError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Storage(_) => "config_storage_failed",
            Self::UnsupportedSchemaVersion(_) => "unsupported_config_schema",
            Self::InvalidToml(_) | Self::InvalidValue(_) => "invalid_config",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileConfigStore {
    data_root: PathBuf,
    config_path: PathBuf,
    lock_path: PathBuf,
}

impl FileConfigStore {
    pub fn new(data_root: impl AsRef<Path>) -> Self {
        let data_root = data_root.as_ref().to_path_buf();
        Self {
            config_path: data_root.join("config.toml"),
            lock_path: data_root.join("config.lock"),
            data_root,
        }
    }

    fn acquire_lock(&self, exclusive: bool) -> Result<File, ConfigError> {
        std::fs::create_dir_all(&self.data_root)
            .map_err(|error| ConfigError::Storage(error.to_string()))?;
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&self.lock_path)
            .map_err(|error| ConfigError::Storage(error.to_string()))?;
        let deadline = Instant::now() + CONFIG_LOCK_TIMEOUT;
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
                    return Err(ConfigError::Storage(
                        "timed out waiting for config lock".to_string(),
                    ));
                }
                Err(error) => return Err(ConfigError::Storage(error.to_string())),
            }
        }
        Ok(file)
    }

    pub fn load(&self) -> Result<AppConfig, ConfigError> {
        let lock = self.acquire_lock(false)?;
        let result = match open_config(&self.config_path) {
            Ok(file) => {
                let metadata = file
                    .metadata()
                    .map_err(|error| ConfigError::Storage(error.to_string()))?;
                if !metadata.is_file() {
                    return Err(ConfigError::Storage(
                        "config.toml must be a regular file".to_string(),
                    ));
                }
                if metadata.len() > MAX_CONFIG_BYTES {
                    return Err(ConfigError::Storage(
                        "config.toml exceeds 1 MiB".to_string(),
                    ));
                }
                let mut bytes = Vec::with_capacity(metadata.len() as usize);
                file.take(MAX_CONFIG_BYTES + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|error| ConfigError::Storage(error.to_string()))?;
                if bytes.len() as u64 > MAX_CONFIG_BYTES {
                    return Err(ConfigError::Storage(
                        "config.toml exceeds 1 MiB".to_string(),
                    ));
                }
                String::from_utf8(bytes)
                    .map_err(|error| ConfigError::Storage(error.to_string()))
                    .and_then(|source| {
                        #[derive(serde::Deserialize)]
                        struct ConfigHeader {
                            schema_version: u32,
                        }
                        let header = toml::from_str::<ConfigHeader>(&source)?;
                        if header.schema_version != 1 {
                            return Err(ConfigError::UnsupportedSchemaVersion(
                                header.schema_version,
                            ));
                        }
                        let mut config = toml::from_str::<AppConfig>(&source)?;
                        config.normalize()?;
                        config.validate()?;
                        Ok(config)
                    })
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(AppConfig::default()),
            Err(error) => Err(ConfigError::Storage(error.to_string())),
        };
        let unlock =
            fs2::FileExt::unlock(&lock).map_err(|error| ConfigError::Storage(error.to_string()));
        result.and_then(|value| unlock.map(|()| value))
    }

    pub fn save(&self, config: &AppConfig) -> Result<(), ConfigError> {
        if config.schema_version != 1 {
            return Err(ConfigError::UnsupportedSchemaVersion(config.schema_version));
        }
        let mut config = config.clone();
        config.normalize()?;
        config.validate()?;
        let lock = self.acquire_lock(true)?;
        let result = (|| {
            let source = toml::to_string_pretty(&config)
                .map_err(|error| ConfigError::Storage(error.to_string()))?;
            let mut temporary = tempfile::NamedTempFile::new_in(&self.data_root)
                .map_err(|error| ConfigError::Storage(error.to_string()))?;
            temporary
                .write_all(source.as_bytes())
                .and_then(|()| temporary.flush())
                .and_then(|()| temporary.as_file().sync_all())
                .map_err(|error| ConfigError::Storage(error.to_string()))?;
            temporary
                .persist(&self.config_path)
                .map_err(|error| ConfigError::Storage(error.error.to_string()))?;
            #[cfg(unix)]
            File::open(&self.data_root)
                .and_then(|directory| directory.sync_all())
                .map_err(|error| ConfigError::Storage(error.to_string()))?;
            Ok(())
        })();
        let unlock =
            fs2::FileExt::unlock(&lock).map_err(|error| ConfigError::Storage(error.to_string()));
        result.and(unlock)
    }
}
