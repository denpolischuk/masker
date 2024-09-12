use std::pin::Pin;

use crate::masker::{
    error::{ConfigParseError, ConfigParseErrorKind},
    transformer::{FirstNameTransformer, LastNameTransformer, TemplateTransformer},
};

use super::TransformerError;

pub enum GeneratedValue {
    String(String),
    Number(String),
}

pub trait Transformer: Sync + Send {
    fn generate(&self, options: &Options) -> Result<GeneratedValue, TransformerError>;
}

pub struct Options {
    pub pk: Box<dyn ToString>,
}

pub fn new_from_yaml(
    yaml: &serde_yaml::Value,
) -> Result<Pin<Box<dyn Transformer>>, ConfigParseError> {
    match yaml["kind"].as_str() {
        Some(s) => match s {
            "FirstName" => Ok(Box::pin(FirstNameTransformer::new())),
            "LastName" => Ok(Box::pin(LastNameTransformer::new())),
            "Template" => Ok(Box::pin(TemplateTransformer::new_from_yaml(yaml)?)),
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
        transformer::new_from_yaml,
    };

    #[test]
    fn get_transformer_from_yaml() -> Result<(), ConfigParseError> {
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
            .expect("expected to get error on parse, got transformer instead");
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
            .expect("expected to get error on parse, got transformer instead");
        assert_eq!(
            err,
            ConfigParseError {
                field: String::from("kind"),
                kind: ConfigParseErrorKind::MissingField,
            }
        )
    }
}
