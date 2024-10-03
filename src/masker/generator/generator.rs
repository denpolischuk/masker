use std::{collections::HashMap, fmt::Display, str::FromStr};

use fake::{locales::EN, Fake};

use crate::masker::{
    error::{ConfigParseError, ConfigParseErrorKind},
    generator::TemplatedGenerator,
    FieldKind,
};

use super::{GeneratorError, IbanGenerator, SimpleGenerator};

#[non_exhaustive]
pub enum GeneratedValue {
    String(String),
    Number(String),
}

impl Display for GeneratedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            GeneratedValue::Number(n) => write!(f, "{n}"),
            GeneratedValue::String(s) => write!(f, "'{s}'"),
        }
    }
}

pub trait Generator: Sync + Send {
    fn generate(&self, options: &Options) -> Result<GeneratedValue, GeneratorError>;
}

pub type Options<'a> = HashMap<&'a String, GeneratedValue>;

pub fn new_from_yaml(
    yaml: &serde_yaml::Value,
) -> Result<(FieldKind, Box<dyn Generator>), ConfigParseError> {
    use fake::faker::address::raw::*;
    use fake::faker::name::raw::*;
    match yaml["kind"].as_str() {
        Some(s) => match FieldKind::from_str(s).unwrap() {
            FieldKind::FirstName => Ok((
                FieldKind::FirstName,
                Box::new(SimpleGenerator::new(|_: &Options| {
                    Ok(GeneratedValue::String(FirstName(EN).fake()))
                })),
            )),
            FieldKind::LastName => Ok((
                FieldKind::LastName,
                Box::new(SimpleGenerator::new(|_: &Options| {
                    Ok(GeneratedValue::String(LastName(EN).fake()))
                })),
            )),
            FieldKind::CityName => Ok((
                FieldKind::CityName,
                Box::new(SimpleGenerator::new(|_: &Options| {
                    Ok(GeneratedValue::String(CityName(EN).fake()))
                })),
            )),
            FieldKind::CountryCode => Ok((
                FieldKind::CountryCode,
                Box::new(SimpleGenerator::new(|_: &Options| {
                    Ok(GeneratedValue::String(CountryCode(EN).fake()))
                })),
            )),
            FieldKind::CountryName => Ok((
                FieldKind::CountryName,
                Box::new(SimpleGenerator::new(|_: &Options| {
                    Ok(GeneratedValue::String(CountryName(EN).fake()))
                })),
            )),
            FieldKind::PostCode => Ok((
                FieldKind::PostCode,
                Box::new(SimpleGenerator::new(|_: &Options| {
                    Ok(GeneratedValue::String(PostCode(EN).fake()))
                })),
            )),
            FieldKind::StateName => Ok((
                FieldKind::StateName,
                Box::new(SimpleGenerator::new(|_: &Options| {
                    Ok(GeneratedValue::String(StateName(EN).fake()))
                })),
            )),
            FieldKind::Template => Ok((
                FieldKind::Template,
                Box::new(TemplatedGenerator::new_from_yaml(yaml)?),
            )),
            FieldKind::Iban => Ok((
                FieldKind::Iban,
                Box::new(IbanGenerator::new_from_yaml(yaml)?),
            )),
            FieldKind::Unknown(field) => Err(ConfigParseError {
                field: s.to_string(),
                kind: ConfigParseErrorKind::UnknownField(field),
            }),
        },
        None => Err(ConfigParseError {
            field: String::from("kind"),
            kind: ConfigParseErrorKind::MissingField,
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::masker::{
        error::{ConfigParseError, ConfigParseErrorKind},
        generator::new_from_yaml,
    };

    #[test]
    fn get_generator_from_yaml() -> Result<(), ConfigParseError> {
        let yaml = serde_yaml::from_str("kind: LastName").unwrap();
        _ = new_from_yaml(&yaml)?;
        Ok(())
    }

    #[test]
    fn fail_on_unknown_kind() {
        let field = "SomethingElse";
        let yaml = serde_yaml::from_str(format!("kind: {field}").as_str()).unwrap();
        let err = new_from_yaml(&yaml)
            .err()
            .expect("expected to get error on parse, got generator instead");
        assert_eq!(
            err,
            ConfigParseError {
                field: String::from(field),
                kind: ConfigParseErrorKind::UnknownField(String::from(field)),
            }
        )
    }
    #[test]
    fn fail_on_missing_kind() {
        let yaml = serde_yaml::from_str("some_other_key: SomethingElse").unwrap();
        let err = new_from_yaml(&yaml)
            .err()
            .expect("expected to get error on parse, got generator instead");
        assert_eq!(
            err,
            ConfigParseError {
                field: String::from("kind"),
                kind: ConfigParseErrorKind::MissingField,
            }
        )
    }
}
