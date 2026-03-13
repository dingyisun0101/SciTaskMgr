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
    /// Maximum number of threads one task may use for its own internal work.
    #[serde(alias = "NUM_THREADS")]
    pub num_threads: usize,
    /// Optional number of task instances requested by the user.
    #[serde(default)]
    pub num_tasks: Option<usize>,
    /// Optional cap on how many tasks may advance concurrently within one group epoch.
    #[serde(default, alias = "NUM_TASK_THREADS")]
    pub num_task_threads: Option<usize>,
    /// Optional manager-level epoch cap.
    #[serde(default)]
    pub max_epochs: Option<u64>,
}

/// Generic directories used by the manager.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IoConfig {
    /// Base directory for task-owned outputs and trajectories.
    pub task_dir: String,
}

impl IoConfig {
    /// Return the trajectory directory implied by `task_dir`.
    pub fn trajectory_dir(&self) -> String {
        format!("{}/trajectories", self.task_dir.trim_end_matches('/'))
    }
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
