mod error;
mod load;
mod types;
mod validate;

/// Error type returned while reading or validating task-group config.
pub use error::ConfigError;
/// Load a task-group config from TOML and load task-owned TOML configs.
pub use load::{load_config, load_task_config};
pub use types::{IoConfig, ManagerConfig, ProgressConfig, RunConfig, TaskGroupConfig};
/// Validate a parsed task-group config against the current envelope contract.
pub use validate::validate_config;

/// Version of the supported manager-owned task-group config envelope.
pub const SUPPORTED_SCHEMA_VERSION: u64 = 1;
