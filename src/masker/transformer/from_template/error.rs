use std::{error::Error, fmt::Display};

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct TemplateParserError {
    pub kind: TemplateParserErrorKind,
}

impl Display for TemplateParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            TemplateParserErrorKind::FailedToParseTemplate(template, start_at) => {
                write!(
                    f,
                    "couldn't parse template {template}: couldn't find the end of sequence that was started at {start_at}",
                )
            }
            TemplateParserErrorKind::UnexpectedToken(template, start_at, token) => {
                write!(
                    f,
                    "couldn't parse template {template}: unexpected token {token} at position {start_at}",
                )
            }
            TemplateParserErrorKind::UnrecognizedSequenceSymbol(template, start_at, token) => {
                write!(
                    f,
                    "couldn't parse template {template}: unrecognized sequence symbol {token} at position {start_at}",
                )
            }
            TemplateParserErrorKind::FailedToResolveValueFromTemplate(template, variable_name) => {
                write!(
                    f,
                    "couldn't parse template {template}: failed to resolve value {variable_name}",
                )
            }
        }
    }
}

impl TemplateParserError {
    pub fn new(kind: TemplateParserErrorKind) -> Self {
        Self { kind }
    }
}

impl Error for TemplateParserError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum TemplateParserErrorKind {
    FailedToParseTemplate(String, usize),
    UnexpectedToken(String, usize, char),
    UnrecognizedSequenceSymbol(String, usize, char),
    FailedToResolveValueFromTemplate(String, String),
}
