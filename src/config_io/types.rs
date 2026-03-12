use std::path::Path;

use serde::{Deserialize, Serialize};
use toml::Value;

use super::{ConfigError, load_config, validate_config};

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
        validate_config(self)
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
