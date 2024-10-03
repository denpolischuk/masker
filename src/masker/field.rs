use std::{cmp::Ordering, fmt::Display, str::FromStr};

use crate::masker::generator::{new_from_yaml, Generator, Options};

use super::{
    error::{ConfigParseError, ConfigParseErrorKind},
    generator::{GeneratedValue, GeneratorError},
};

#[derive(PartialEq, Eq, Clone)]
pub enum FieldKind {
    FirstName,
    LastName,
    CityName,
    CountryCode,
    CountryName,
    PostCode,
    StateName,
    Template,
    Iban,
    Unknown(String),
}

impl FromStr for FieldKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "FirstName" => Ok(Self::FirstName),
            "LastName" => Ok(Self::LastName),
            "CityName" => Ok(Self::CityName),
            "CountryCode" => Ok(Self::CountryCode),
            "CountryName" => Ok(Self::CountryName),
            "PostCode" => Ok(Self::PostCode),
            "StateName" => Ok(Self::StateName),
            "Template" => Ok(Self::Template),
            "Iban" => Ok(Self::Iban),
            _ => Ok(Self::Unknown(s.to_string())),
        }
    }
}

pub struct Field {
    field_name: String,
    pub kind: FieldKind,
    generator: Box<dyn Generator>,
}

impl Field {
    pub fn new(field_name: String, kind: FieldKind, generator: Box<dyn Generator>) -> Self {
        Self {
            field_name,
            kind,
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
        let (kind, generator) = new_from_yaml(yaml)?;
        Ok(Self::new(name, kind, generator))
    }

    pub fn get_column_name(&self) -> &String {
        &self.field_name
    }

    // This function is needed for comparing the fields in terms of simple and composed
    // generators.
    fn has_composed_generator_kind(&self) -> bool {
        matches!(self.kind, FieldKind::Template)
    }
}

impl Eq for Field {}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        self.field_name == other.field_name && self.kind == other.kind
    }
}

impl PartialOrd for Field {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Field {
    fn cmp(&self, other: &Self) -> Ordering {
        let is_complex_gen_self = self.has_composed_generator_kind();
        let is_complex_gen_other = other.has_composed_generator_kind();
        match is_complex_gen_self.cmp(&is_complex_gen_other) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.field_name.cmp(&other.field_name) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        Ordering::Equal
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "field_name: {}", self.field_name)
    }
}
