use std::error::Error;
use std::fmt::{Display, Formatter};

/// Errors that can occur while loading or validating manager config.
#[derive(Debug)]
pub enum ConfigError {
    /// Reading the config file from disk failed.
    Io(std::io::Error),
    /// The config file extension is not supported by this loader.
    UnsupportedExtension(String),
    /// TOML syntax was invalid or could not be deserialized.
    ParseToml(toml::de::Error),
    /// The declared schema version does not match the supported version.
    UnsupportedSchemaVersion { found: u64, supported: u64 },
    /// A required field was missing or contained an invalid value.
    InvalidField(&'static str, String),
}

impl Display for ConfigError {
    /// Format the error as a user-facing message.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "failed to read config: {err}"),
            Self::UnsupportedExtension(ext) => {
                write!(f, "unsupported config extension `{ext}`; use `.toml`")
            }
            Self::ParseToml(err) => write!(f, "failed to parse TOML config: {err}"),
            Self::UnsupportedSchemaVersion { found, supported } => write!(
                f,
                "unsupported schema_version `{found}`; supported schema_version is `{supported}`"
            ),
            Self::InvalidField(field, message) => write!(f, "invalid field `{field}`: {message}"),
        }
    }
}

impl Error for ConfigError {
    /// Return the wrapped source error when one exists.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::ParseToml(err) => Some(err),
            Self::UnsupportedExtension(_)
            | Self::UnsupportedSchemaVersion { .. }
            | Self::InvalidField(_, _) => None,
        }
    }
}
