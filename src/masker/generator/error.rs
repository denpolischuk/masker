use std::{error::Error, fmt::Display};

use super::{from_template::TemplatedParserError, Generator};

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct GeneratorError {
    pub generator_name: String,
    pub kind: GeneratorErrorKind,
}

impl Display for GeneratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            GeneratorErrorKind::FailedToParseTemplatedGenerator(_) => {
                write!(
                    f,
                    "couldn't parse template for generator {}",
                    self.generator_name
                )
            }
        }
    }
}

impl GeneratorError {
    pub fn new<T>(kind: GeneratorErrorKind) -> Self
    where
        T: Sized + Generator,
    {
        Self {
            kind,
            generator_name: String::from(std::any::type_name::<T>()),
        }
    }
}

impl Error for GeneratorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum GeneratorErrorKind {
    FailedToParseTemplatedGenerator(TemplatedParserError),
}
