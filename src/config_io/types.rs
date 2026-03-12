use std::path::Path;

use serde::{Deserialize, Serialize};
use toml::Value;

use super::{ConfigError, load_config, validate_config};

/// Top-level manager-owned config envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManagerConfig {
    /// Version of the manager envelope schema expected by this crate.
    pub schema_version: u64,
    /// Run-level metadata used by the manager.
    pub run: RunConfig,
    /// Manager-owned output and filesystem locations.
    pub io: IoConfig,
    /// Generic checkpoint and resume policy.
    pub checkpoint: CheckpointConfig,
    /// Optional manager-owned progress settings.
    #[serde(default)]
    pub progress: Option<ProgressConfig>,
    /// Task-owned payload left opaque to the manager.
    pub task: Value,
}

impl ManagerConfig {
    /// Validate this parsed config against the manager envelope rules.
    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_config(self)
    }

    /// Load and validate a config directly from a file path.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        load_config(path)
    }
}

/// Universal run metadata needed before task-specific parsing begins.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunConfig {
    /// Human-readable name for the run.
    pub name: String,
    /// Identifier used to select the concrete task implementation.
    pub task_type: String,
    /// Optional number of task instances requested by the user.
    #[serde(default)]
    pub num_tasks: Option<usize>,
    /// Optional manager-level epoch cap.
    #[serde(default)]
    pub max_epochs: Option<u64>,
}

/// Generic directories used by the manager.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IoConfig {
    /// Base output directory for the run.
    pub root_dir: String,
    /// Optional directory for trajectory files.
    #[serde(default)]
    pub trajectory_dir: Option<String>,
    /// Optional directory for checkpoint files.
    #[serde(default)]
    pub checkpoint_dir: Option<String>,
}

/// Generic checkpoint policy that applies regardless of task type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckpointConfig {
    /// Resume behavior when existing checkpoints are present.
    pub resume: ResumePolicy,
    /// Whether known-invalid checkpoint files should be removed automatically.
    #[serde(default)]
    pub cleanup_invalid: bool,
    /// Whether grouped tasks should be synchronized to a shared epoch.
    #[serde(default)]
    pub sync_group_epochs: bool,
}

/// Resume policy understood by the manager.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResumePolicy {
    /// Ignore existing checkpoints and always start from scratch.
    Never,
    /// Resume if checkpoints exist, otherwise start fresh.
    IfAvailable,
    /// Require checkpoints to exist before the run may start.
    Require,
}

/// Optional generic progress settings owned by the manager.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgressConfig {
    /// Whether manager-level progress reporting should be enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Optional refresh frequency for future live progress consumers.
    #[serde(default)]
    pub refresh_hz: Option<u64>,
}
