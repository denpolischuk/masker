use std::{env::VarError, error::Error, fmt::Display};

use super::transformer::TransformerError;

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct ConfigParseError {
    pub field: String,
    pub kind: ConfigParseErrorKind,
}

impl Display for ConfigParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ConfigParseErrorKind::MissingField => {
                write!(f, "field {} is missing in the config", self.field)
            }
            ConfigParseErrorKind::UnexpectedFieldValue(s) => {
                write!(f, "field {} has unexpected value {}", self.field, s)
            }
            ConfigParseErrorKind::UnknownField(s) => {
                write!(f, "field {} is unknown but has been found in config", s)
            }
            ConfigParseErrorKind::UnexpectedFieldType => {
                write!(
                    f,
                    "field {} is of unexpected type and the value couldn't be parsed",
                    self.field
                )
            }
            ConfigParseErrorKind::FailedToReadValueFromEnv(key, _) => {
                write!(
                    f,
                    "couldn't read value from env by key {key} for field {}",
                    self.field
                )
            }
            ConfigParseErrorKind::FailedToCreateTransformerFromConfig(_) => {
                write!(f, "couldn't parse transformer for field {}", self.field)
            }
        }
    }
}

impl Error for ConfigParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            ConfigParseErrorKind::FailedToReadValueFromEnv(_, e) => e.source(),
            ConfigParseErrorKind::FailedToCreateTransformerFromConfig(e) => e.source(),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum ConfigParseErrorKind {
    MissingField,
    UnexpectedFieldValue(String),
    UnknownField(String),
    UnexpectedFieldType,
    FailedToReadValueFromEnv(String, VarError),
    FailedToCreateTransformerFromConfig(TransformerError),
}
