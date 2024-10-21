use std::{error::Error, fmt::Display};

use chrono::ParseError;

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
            GeneratorErrorKind::ParseTemplatedGenerator(_) => {
                write!(
                    f,
                    "couldn't parse template for generator {}",
                    self.generator_name
                )
            }
            GeneratorErrorKind::GenerateIban => {
                write!(f, "couldn't generate iban")
            }
            GeneratorErrorKind::GenerateIbanForCountryCode(code) => {
                write!(f, "couldn't generate iban for country code {}", code)
            }
            GeneratorErrorKind::UnexpectedCountryCodeForIban(code) => {
                write!(f, "unexpected country code for iban - {}", code)
            }
            GeneratorErrorKind::WrongDateTimeFormatForDateGenerator(_) => {
                write!(f, "couldn't parse date")
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
        match &self.kind {
            GeneratorErrorKind::WrongDateTimeFormatForDateGenerator(err) => err.source(),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum GeneratorErrorKind {
    GenerateIban,
    GenerateIbanForCountryCode(String),
    ParseTemplatedGenerator(TemplatedParserError),
    UnexpectedCountryCodeForIban(String),
    WrongDateTimeFormatForDateGenerator(ParseError),
}
