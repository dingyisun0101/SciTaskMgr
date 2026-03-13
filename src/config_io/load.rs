use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;

use super::{ConfigError, TaskGroupConfig};

/// Read, parse, and validate a manager config from a TOML file path.
pub fn load_config(path: impl AsRef<Path>) -> Result<TaskGroupConfig, ConfigError> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(ConfigError::Io)?;
    let config: TaskGroupConfig = load_toml_file(&raw, path)?;
    config.validate()?;
    Ok(config)
}

/// Read and parse a task-owned TOML config from disk.
pub fn load_task_config<T>(path: impl AsRef<Path>) -> Result<T, ConfigError>
where
    T: DeserializeOwned,
{
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(ConfigError::Io)?;
    load_toml_file(&raw, path)
}

/// Parse one TOML file into the requested type after checking its extension.
fn load_toml_file<T>(raw: &str, path: &Path) -> Result<T, ConfigError>
where
    T: DeserializeOwned,
{
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| ConfigError::UnsupportedExtension(String::new()))?;

    match extension {
        "toml" => toml::from_str(raw).map_err(ConfigError::ParseToml),
        other => return Err(ConfigError::UnsupportedExtension(other.to_string())),
    }
}
