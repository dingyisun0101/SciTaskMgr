use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{ConfigError, load_config, validate_config};

/// Top-level task-group config owned by the manager.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskGroupConfig {
    /// Version of the manager envelope schema expected by this crate.
    pub schema_version: u64,
    /// Run-level metadata used by the manager.
    pub run: RunConfig,
    /// Manager-owned output and filesystem locations.
    pub io: IoConfig,
    /// Optional manager-owned progress settings.
    #[serde(default)]
    pub progress: Option<ProgressConfig>,
}

impl TaskGroupConfig {
    /// Validate this parsed config against the manager envelope rules.
    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_config(self)
    }

    /// Load and validate a config directly from a file path.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        load_config(path)
    }
}

/// Backward-compatible alias for the manager-owned task-group config.
pub type ManagerConfig = TaskGroupConfig;

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
    /// Base directory for one managed task group.
    pub task_group_dir: String,
}

impl IoConfig {
    /// Return the task directory derived for one task index.
    pub fn task_dir(&self, task_index: usize) -> String {
        format!(
            "{}/tasks/task_{task_index:04}",
            self.task_group_dir.trim_end_matches('/')
        )
    }

    /// Return the trajectory directory derived for one task index.
    pub fn trajectory_dir(&self, task_index: usize) -> String {
        format!("{}/trajectories", self.task_dir(task_index))
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
