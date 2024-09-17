use std::{error::Error, fmt::Display};

use super::{from_template::TemplateParserError, Transformer};

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct TransformerError {
    pub transformer_name: String,
    pub kind: TransformerErrorKind,
}

impl Display for TransformerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            TransformerErrorKind::FailedToParseTempalteTransformer(_) => {
                write!(
                    f,
                    "couldn't parse template for transformer {}",
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
    FailedToParseTempalteTransformer(TemplateParserError),
}
