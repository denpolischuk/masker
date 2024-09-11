use std::{error::Error, fmt::Display};

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
        }
    }
}

impl Error for ConfigParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum ConfigParseErrorKind {
    MissingField,
    UnexpectedFieldValue(String),
    UnknownField(String),
}