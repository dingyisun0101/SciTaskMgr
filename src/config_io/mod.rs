use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SUPPORTED_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManagerConfig {
    pub schema_version: u64,
    pub run: RunConfig,
    pub io: IoConfig,
    pub checkpoint: CheckpointConfig,
    #[serde(default)]
    pub progress: Option<ProgressConfig>,
    pub task: Value,
}

impl ManagerConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.schema_version != SUPPORTED_SCHEMA_VERSION {
            return Err(ConfigError::UnsupportedSchemaVersion {
                found: self.schema_version,
                supported: SUPPORTED_SCHEMA_VERSION,
            });
        }

        if self.run.name.trim().is_empty() {
            return Err(ConfigError::InvalidField(
                "run.name",
                "run name must not be empty".to_string(),
            ));
        }

        if self.run.task_type.trim().is_empty() {
            return Err(ConfigError::InvalidField(
                "run.task_type",
                "task type must not be empty".to_string(),
            ));
        }

        if self.io.root_dir.trim().is_empty() {
            return Err(ConfigError::InvalidField(
                "io.root_dir",
                "root dir must not be empty".to_string(),
            ));
        }

        if let Some(progress) = &self.progress
            && progress.enabled
            && progress.refresh_hz == Some(0)
        {
            return Err(ConfigError::InvalidField(
                "progress.refresh_hz",
                "refresh_hz must be greater than zero when progress is enabled".to_string(),
            ));
        }

        Ok(())
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        load_config(path)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunConfig {
    pub name: String,
    pub task_type: String,
    #[serde(default)]
    pub num_tasks: Option<usize>,
    #[serde(default)]
    pub max_epochs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IoConfig {
    pub root_dir: String,
    #[serde(default)]
    pub trajectory_dir: Option<String>,
    #[serde(default)]
    pub checkpoint_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckpointConfig {
    pub resume: ResumePolicy,
    #[serde(default)]
    pub cleanup_invalid: bool,
    #[serde(default)]
    pub sync_group_epochs: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResumePolicy {
    Never,
    IfAvailable,
    Require,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgressConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub refresh_hz: Option<u64>,
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    UnsupportedExtension(String),
    ParseToml(toml::de::Error),
    ParseJson(serde_json::Error),
    UnsupportedSchemaVersion { found: u64, supported: u64 },
    InvalidField(&'static str, String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "failed to read config: {err}"),
            Self::UnsupportedExtension(ext) => {
                write!(f, "unsupported config extension `{ext}`; use `.toml` or `.json`")
            }
            Self::ParseToml(err) => write!(f, "failed to parse TOML config: {err}"),
            Self::ParseJson(err) => write!(f, "failed to parse JSON config: {err}"),
            Self::UnsupportedSchemaVersion { found, supported } => write!(
                f,
                "unsupported schema_version `{found}`; supported schema_version is `{supported}`"
            ),
            Self::InvalidField(field, message) => write!(f, "invalid field `{field}`: {message}"),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::ParseToml(err) => Some(err),
            Self::ParseJson(err) => Some(err),
            Self::UnsupportedExtension(_)
            | Self::UnsupportedSchemaVersion { .. }
            | Self::InvalidField(_, _) => None,
        }
    }
}

pub fn load_config(path: impl AsRef<Path>) -> Result<ManagerConfig, ConfigError> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(ConfigError::Io)?;
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| ConfigError::UnsupportedExtension(String::new()))?;

    let config = match extension {
        "toml" => toml::from_str(&raw).map_err(ConfigError::ParseToml)?,
        "json" => serde_json::from_str(&raw).map_err(ConfigError::ParseJson)?,
        other => return Err(ConfigError::UnsupportedExtension(other.to_string())),
    };

    config.validate()?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_toml_envelope() {
        let raw = r#"
schema_version = 1

[run]
name = "screening"
task_type = "dses_screening"
num_tasks = 50

[io]
root_dir = "/tmp/out"

[checkpoint]
resume = "if_available"
cleanup_invalid = true
sync_group_epochs = false

[progress]
enabled = true
refresh_hz = 5

[task]
mission = "b2"
steps = [3, 4]
"#;

        let config: ManagerConfig = toml::from_str(raw).expect("config should parse");
        config.validate().expect("config should validate");

        assert_eq!(config.schema_version, SUPPORTED_SCHEMA_VERSION);
        assert_eq!(config.run.task_type, "dses_screening");
        assert_eq!(config.task["mission"], "b2");
    }

    #[test]
    fn rejects_wrong_schema_version() {
        let config = ManagerConfig {
            schema_version: 2,
            run: RunConfig {
                name: "run".to_string(),
                task_type: "demo".to_string(),
                num_tasks: None,
                max_epochs: None,
            },
            io: IoConfig {
                root_dir: "/tmp/out".to_string(),
                trajectory_dir: None,
                checkpoint_dir: None,
            },
            checkpoint: CheckpointConfig {
                resume: ResumePolicy::IfAvailable,
                cleanup_invalid: false,
                sync_group_epochs: false,
            },
            progress: None,
            task: Value::Null,
        };

        let err = config.validate().expect_err("schema version should fail");
        assert!(matches!(
            err,
            ConfigError::UnsupportedSchemaVersion {
                found: 2,
                supported: 1
            }
        ));
    }
}
