use super::token::Token;
use crate::masker::{
    error::{ConfigParseError, ConfigParseErrorKind},
    transformer::{
        error::TransformerErrorKind, GeneratedValue, Options, Transformer, TransformerError,
    },
};

pub struct TemplateTransformer {
    template: String,
    tokens: Vec<Token>,
}

impl TemplateTransformer {
    pub fn new_from_yaml(yaml: &serde_yaml::Value) -> Result<Self, ConfigParseError> {
        let field = "template";
        match yaml[field].as_str() {
            Some(t) => {
                let tokens = Token::parse_tokens_from_template(&t.to_string()).map_err(|e| {
                    ConfigParseError {
                        field: field.to_string(),
                        kind: ConfigParseErrorKind::FailedToCreateTransformerFromConfig(
                            TransformerError::new::<Self>(
                                TransformerErrorKind::FailedToParseTempalteTransformer(e),
                            ),
                        ),
                    }
                })?;
                Ok(Self {
                    template: t.to_string(),
                    tokens,
                })
            }
            None => Err(ConfigParseError {
                field: "template".to_string(),
                kind: ConfigParseErrorKind::MissingField,
            }),
        }
    }
}

impl Transformer for TemplateTransformer {
    fn generate(&self, opts: &Options) -> Result<GeneratedValue, TransformerError> {
        let res = self.template.replace("{id}", opts.pk.to_string().as_str());

        Ok(GeneratedValue::String(res))
    }
}
