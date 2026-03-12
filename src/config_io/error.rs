use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    UnsupportedExtension(String),
    ParseToml(toml::de::Error),
    UnsupportedSchemaVersion { found: u64, supported: u64 },
    InvalidField(&'static str, String),
}

impl Display for ConfigError {
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
