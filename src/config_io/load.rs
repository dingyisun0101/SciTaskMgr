use std::fs;
use std::path::Path;

use super::{ConfigError, ManagerConfig};

pub fn load_config(path: impl AsRef<Path>) -> Result<ManagerConfig, ConfigError> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(ConfigError::Io)?;
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| ConfigError::UnsupportedExtension(String::new()))?;

    let config: ManagerConfig = match extension {
        "toml" => toml::from_str(&raw).map_err(ConfigError::ParseToml)?,
        other => return Err(ConfigError::UnsupportedExtension(other.to_string())),
    };

    config.validate()?;
    Ok(config)
}
