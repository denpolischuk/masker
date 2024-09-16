use std::{error::Error, fmt::Display};

use super::Transformer;

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct TransformerError {
    pub transformer_name: String,
    pub kind: TransformerErrorKind,
}

impl Display for TransformerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            TransformerErrorKind::FailedToParseTemplate(template, start_at) => {
                write!(
                    f,
                    "transformer {} couldn't parse template {template}: couldn't find the end of sequence that was started at {start_at}",
                    self.transformer_name
                )
            }
            TransformerErrorKind::UnexpectedToken(template, start_at, token) => {
                write!(
                    f,
                    "transformer {} couldn't parse template {template}: unexpected token {token} at position {start_at}",
                    self.transformer_name
                )
            }
        }
    }
}

impl TransformerError {
    pub fn new<T>(kind: TransformerErrorKind) -> Self
    where
        T: Sized + Transformer,
    {
        Self {
            kind,
            transformer_name: String::from(std::any::type_name::<T>()),
        }
    }
}

impl Error for TransformerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum TransformerErrorKind {
    FailedToParseTemplate(String, usize),
    UnexpectedToken(String, usize, char),
}
