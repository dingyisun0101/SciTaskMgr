mod error;
mod load;
mod types;
mod validate;

/// Error type returned while reading or validating manager config.
pub use error::ConfigError;
/// Load a manager config from a TOML file and validate the envelope.
pub use load::load_config;
pub use types::{IoConfig, ManagerConfig, ProgressConfig, RunConfig};
/// Validate a parsed manager config against the current envelope contract.
pub use validate::validate_config;

/// Version of the supported manager-owned config envelope.
pub const SUPPORTED_SCHEMA_VERSION: u64 = 1;
