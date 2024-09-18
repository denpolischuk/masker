use rand::{distributions::Uniform, thread_rng, Rng};

use super::{
    token::{Token, TokenKind},
    TemplateParserError,
};
use crate::masker::{
    error::{ConfigParseError, ConfigParseErrorKind},
    transformer::{
        error::TransformerErrorKind, GeneratedValue, Options, Transformer, TransformerError,
    },
};

pub struct TemplateTransformer {
    upper_case_letters_set: Uniform<char>,
    lower_case_letters_set: Uniform<char>,
    digits_set: Uniform<char>,
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
                    upper_case_letters_set: Uniform::new(char::from(0x41), char::from(0x5a)),
                    lower_case_letters_set: Uniform::new(char::from(0x61), char::from(0x7a)),
                    digits_set: Uniform::new(char::from(0x30), char::from(0x39)),
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
        let resolved: Result<Vec<String>, TransformerError> = self
            .tokens
            .iter()
            .map(|token| match &token.0 {
                TokenKind::Plain(s) => Ok(s.clone()),
                TokenKind::Variable(v) => match opts.get(v) {
                    Some(val) => Ok(val.to_string()),
                    None => {
                        Err(
                            TransformerError::new::<Self>(
                                TransformerErrorKind::FailedToParseTempalteTransformer(TemplateParserError::new(super::error::TemplateParserErrorKind::FailedToResolveValueFromTemplate(self.template.clone(), v.clone())))
                            )
                        )
                    },
                },
                TokenKind::CapitalLetterSeq(seq) => Ok(thread_rng().sample_iter(&self.upper_case_letters_set).take(seq.chars().count()).collect::<String>()),
                TokenKind::LowerCaseLetterSeq(seq) => Ok(thread_rng().sample_iter(&self.lower_case_letters_set).take(seq.chars().count()).collect::<String>()),
                TokenKind::DigitSeq(seq) => Ok(thread_rng().sample_iter(&self.digits_set).take(seq.chars().count()).collect::<String>()),
            })
            .collect();

        Ok(GeneratedValue::String(resolved?.join("")))
    }
}
