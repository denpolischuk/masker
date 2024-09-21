use std::collections::HashMap;

use crate::masker::{
    error::{ConfigParseError, ConfigParseErrorKind},
    generator::{FirstNameGenerator, LastNameGenerator, TemplatedGenerator},
};

use super::GeneratorError;

pub enum GeneratedValue {
    String(String),
    Number(String),
}

pub trait Generator: Sync + Send {
    fn generate(&self, options: &Options) -> Result<GeneratedValue, GeneratorError>;
}

pub type Options<'a> = HashMap<&'a String, &'a String>;

pub fn new_from_yaml(yaml: &serde_yaml::Value) -> Result<Box<dyn Generator>, ConfigParseError> {
    match yaml["kind"].as_str() {
        Some(s) => match s {
            "FirstName" => Ok(Box::new(FirstNameGenerator::new())),
            "LastName" => Ok(Box::new(LastNameGenerator::new())),
            "Template" => Ok(Box::new(TemplatedGenerator::new_from_yaml(yaml)?)),
            "MobilePhone" => todo!(),
            field => Err(ConfigParseError {
                field: s.to_string(),
                kind: ConfigParseErrorKind::UnknownField(String::from(field)),
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
