use super::{ConfigError, SUPPORTED_SCHEMA_VERSION, TaskGroupConfig};

/// Validate a parsed config against the manager-owned envelope contract.
pub fn validate_config(config: &TaskGroupConfig) -> Result<(), ConfigError> {
    if config.schema_version != SUPPORTED_SCHEMA_VERSION {
        return Err(ConfigError::UnsupportedSchemaVersion {
            found: config.schema_version,
            supported: SUPPORTED_SCHEMA_VERSION,
        });
    }

    if config.run.name.trim().is_empty() {
        return Err(ConfigError::InvalidField(
            "run.name",
            "run name must not be empty".to_string(),
        ));
    }

    if config.run.task_type.trim().is_empty() {
        return Err(ConfigError::InvalidField(
            "run.task_type",
            "task type must not be empty".to_string(),
        ));
    }

    if config.run.num_threads == 0 {
        return Err(ConfigError::InvalidField(
            "run.num_threads",
            "num_threads must be greater than zero".to_string(),
        ));
    }

    if config.run.num_task_threads == Some(0) {
        return Err(ConfigError::InvalidField(
            "run.num_task_threads",
            "num_task_threads must be greater than zero when provided".to_string(),
        ));
    }

    if config.io.task_group_dir.trim().is_empty() {
        return Err(ConfigError::InvalidField(
            "io.task_group_dir",
            "task_group_dir must not be empty".to_string(),
        ));
    }

    if let Some(progress) = &config.progress
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
