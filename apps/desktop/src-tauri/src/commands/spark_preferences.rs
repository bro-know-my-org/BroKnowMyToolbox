use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::DesktopState;

const PREFERENCES_LOCK_TIMEOUT: Duration = Duration::from_millis(500);
const LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(10);
const MAX_PREFERENCES_BYTES: u64 = 64 * 1024;

fn open_preferences(path: &Path) -> std::io::Result<File> {
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

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SparkPreferences {
    pub provider_id: String,
    pub base_url: String,
    pub model: String,
    pub temperature: f64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredSparkPreferences {
    schema_version: u32,
    #[serde(flatten)]
    preferences: SparkPreferences,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SparkPreferencesError {
    pub code: String,
    pub message: String,
}

impl SparkPreferencesError {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

fn storage_directory(state: &DesktopState) -> PathBuf {
    state.data_root.join("tools").join("spark-analyzer")
}

fn lock_file(path: &Path, exclusive: bool) -> Result<File, SparkPreferencesError> {
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)
        .map_err(|error| {
            SparkPreferencesError::new("spark_preferences_storage_failed", error.to_string())
        })?;
    let deadline = Instant::now() + PREFERENCES_LOCK_TIMEOUT;
    loop {
        let result = if exclusive {
            fs2::FileExt::try_lock_exclusive(&file)
        } else {
            fs2::FileExt::try_lock_shared(&file)
        };
        match result {
            Ok(()) => return Ok(file),
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
            {
                std::thread::sleep(LOCK_RETRY_INTERVAL);
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                return Err(SparkPreferencesError::new(
                    "spark_preferences_lock_timeout",
                    "timed out waiting for Spark preferences lock",
                ));
            }
            Err(error) => {
                return Err(SparkPreferencesError::new(
                    "spark_preferences_storage_failed",
                    error.to_string(),
                ));
            }
        }
    }
}

fn validate(preferences: &SparkPreferences) -> Result<(), SparkPreferencesError> {
    let valid_provider = !preferences.provider_id.is_empty()
        && preferences.provider_id.len() <= 64
        && preferences
            .provider_id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    if !valid_provider || preferences.model.len() > 256 {
        return Err(SparkPreferencesError::new(
            "invalid_spark_preferences",
            "invalid provider id or model length",
        ));
    }
    if !preferences.temperature.is_finite() || !(0.0..=2.0).contains(&preferences.temperature) {
        return Err(SparkPreferencesError::new(
            "invalid_spark_preferences",
            "temperature must be between 0 and 2",
        ));
    }
    if preferences.base_url.len() > 2048 {
        return Err(SparkPreferencesError::new(
            "invalid_spark_preferences",
            "provider URL is too long",
        ));
    }
    if !preferences.base_url.is_empty() {
        let url = url::Url::parse(&preferences.base_url).map_err(|error| {
            SparkPreferencesError::new("invalid_spark_preferences", error.to_string())
        })?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(SparkPreferencesError::new(
                "invalid_spark_preferences",
                "provider URL must use HTTP or HTTPS",
            ));
        }
        if !url.username().is_empty() || url.password().is_some() {
            return Err(SparkPreferencesError::new(
                "invalid_spark_preferences",
                "provider URL must not contain credentials",
            ));
        }
    }
    Ok(())
}

pub fn load_spark_preferences(
    state: &DesktopState,
) -> Result<Option<SparkPreferences>, SparkPreferencesError> {
    let directory = storage_directory(state);
    std::fs::create_dir_all(&directory).map_err(|error| {
        SparkPreferencesError::new("spark_preferences_storage_failed", error.to_string())
    })?;
    let lock = lock_file(&directory.join("preferences.lock"), false)?;
    let path = directory.join("preferences.json");
    let result = match open_preferences(&path) {
        Ok(mut file) => {
            let metadata = file.metadata().map_err(|error| {
                SparkPreferencesError::new("spark_preferences_storage_failed", error.to_string())
            })?;
            if !metadata.is_file() {
                return Err(SparkPreferencesError::new(
                    "invalid_spark_preferences",
                    "Spark preferences must be a regular file",
                ));
            }
            if metadata.len() > MAX_PREFERENCES_BYTES {
                Err(SparkPreferencesError::new(
                    "invalid_spark_preferences",
                    "Spark preferences exceed 64 KiB",
                ))
            } else {
                let mut source = Vec::with_capacity(metadata.len() as usize);
                std::io::Read::by_ref(&mut file)
                    .take(MAX_PREFERENCES_BYTES + 1)
                    .read_to_end(&mut source)
                    .map_err(|error| {
                        SparkPreferencesError::new(
                            "spark_preferences_storage_failed",
                            error.to_string(),
                        )
                    })?;
                if source.len() as u64 > MAX_PREFERENCES_BYTES {
                    return Err(SparkPreferencesError::new(
                        "invalid_spark_preferences",
                        "Spark preferences exceed 64 KiB",
                    ));
                }
                let stored: StoredSparkPreferences =
                    serde_json::from_slice(&source).map_err(|error| {
                        SparkPreferencesError::new("invalid_spark_preferences", error.to_string())
                    })?;
                if stored.schema_version != 1 {
                    Err(SparkPreferencesError::new(
                        "unsupported_spark_preferences_schema",
                        format!(
                            "unsupported Spark preferences schema version: {}",
                            stored.schema_version
                        ),
                    ))
                } else {
                    validate(&stored.preferences)?;
                    Ok(Some(stored.preferences))
                }
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(SparkPreferencesError::new(
            "spark_preferences_storage_failed",
            error.to_string(),
        )),
    };
    let unlock = fs2::FileExt::unlock(&lock).map_err(|error| {
        SparkPreferencesError::new("spark_preferences_storage_failed", error.to_string())
    });
    result.and_then(|value| unlock.map(|()| value))
}

pub fn save_spark_preferences(
    state: &DesktopState,
    preferences: SparkPreferences,
) -> Result<(), SparkPreferencesError> {
    validate(&preferences)?;
    let directory = storage_directory(state);
    std::fs::create_dir_all(&directory).map_err(|error| {
        SparkPreferencesError::new("spark_preferences_storage_failed", error.to_string())
    })?;
    let lock = lock_file(&directory.join("preferences.lock"), true)?;
    let result = (|| {
        let mut temporary = tempfile::NamedTempFile::new_in(&directory).map_err(|error| {
            SparkPreferencesError::new("spark_preferences_storage_failed", error.to_string())
        })?;
        serde_json::to_writer_pretty(
            &mut temporary,
            &StoredSparkPreferences {
                schema_version: 1,
                preferences,
            },
        )
        .map_err(|error| {
            SparkPreferencesError::new("spark_preferences_storage_failed", error.to_string())
        })?;
        temporary
            .write_all(b"\n")
            .and_then(|()| temporary.flush())
            .and_then(|()| temporary.as_file().sync_all())
            .map_err(|error| {
                SparkPreferencesError::new("spark_preferences_storage_failed", error.to_string())
            })?;
        temporary
            .persist(directory.join("preferences.json"))
            .map_err(|error| {
                SparkPreferencesError::new(
                    "spark_preferences_storage_failed",
                    error.error.to_string(),
                )
            })?;
        #[cfg(unix)]
        File::open(&directory)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| {
                SparkPreferencesError::new("spark_preferences_storage_failed", error.to_string())
            })?;
        Ok(())
    })();
    let _ = fs2::FileExt::unlock(&lock);
    result
}

#[tauri::command]
pub async fn load_spark_preferences_command(
    state: tauri::State<'_, DesktopState>,
) -> Result<Option<SparkPreferences>, SparkPreferencesError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || load_spark_preferences(&state))
        .await
        .map_err(|error| {
            SparkPreferencesError::new("spark_preferences_task_failed", error.to_string())
        })?
}

#[tauri::command]
pub async fn save_spark_preferences_command(
    state: tauri::State<'_, DesktopState>,
    preferences: SparkPreferences,
) -> Result<(), SparkPreferencesError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || save_spark_preferences(&state, preferences))
        .await
        .map_err(|error| {
            SparkPreferencesError::new("spark_preferences_task_failed", error.to_string())
        })?
}
