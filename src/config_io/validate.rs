use super::{ConfigError, ManagerConfig, SUPPORTED_SCHEMA_VERSION};

pub fn validate_config(config: &ManagerConfig) -> Result<(), ConfigError> {
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

    if config.io.root_dir.trim().is_empty() {
        return Err(ConfigError::InvalidField(
            "io.root_dir",
            "root dir must not be empty".to_string(),
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
