use std::fmt::Display;

use crate::masker::generator::{new_from_yaml, Generator, Options};

use super::{
    error::{ConfigParseError, ConfigParseErrorKind},
    generator::{GeneratedValue, GeneratorError},
};

pub struct Field {
    field_name: String,
    generator: Box<dyn Generator>,
}

impl Field {
    pub fn new(field_name: String, generator: Box<dyn Generator>) -> Self {
        Self {
            field_name,
            generator,
        }
    }

    pub fn generate(&self, opts: &Options) -> Result<GeneratedValue, GeneratorError> {
        self.generator.generate(opts)
    }

    pub fn new_from_yaml(yaml: &serde_yaml::Value) -> Result<Self, ConfigParseError> {
        let field_name = String::from("name");
        let name = match yaml[field_name.as_str()].as_str() {
            Some(s) => String::from(s),
            None => {
                return Err(ConfigParseError {
                    kind: ConfigParseErrorKind::MissingField,
                    field: field_name,
                })
            }
        };
        let generator = new_from_yaml(yaml)?;
        Ok(Self::new(name, generator))
    }

    pub fn get_column_name(&self) -> &String {
        &self.field_name
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "field_name: {}", self.field_name)
    }
}
