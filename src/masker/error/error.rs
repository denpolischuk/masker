use std::{error::Error, fmt::Display};

#[derive(Debug)]
#[non_exhaustive]
pub struct ConfigParseError {
    pub field: String,
    pub kind: ConfigParseErrorKind,
}

impl Display for ConfigParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ConfigParseErrorKind::MissingField => {
                write!(f, "field {} is missing in the config", self.field)
            }
            ConfigParseErrorKind::InvalidFieldType => {
                write!(f, "field {} has invalid type", self.field)
            }
            ConfigParseErrorKind::UnknownField => {
                write!(
                    f,
                    "field {} is unknown but has been found in config",
                    self.field
                )
            }
            _ => write!(f, "unkown error with field {}", self.field),
        }
    }
}

impl Error for ConfigParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ConfigParseErrorKind {
    MissingField,
    InvalidFieldType,
    UnknownField,
}
