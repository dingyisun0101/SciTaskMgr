mod error;
mod load;
mod types;
mod validate;

pub use error::ConfigError;
pub use load::load_config;
pub use types::{
    CheckpointConfig, IoConfig, ManagerConfig, ProgressConfig, ResumePolicy, RunConfig,
};
pub use validate::validate_config;

pub const SUPPORTED_SCHEMA_VERSION: u64 = 1;
