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
        let mut res = String::new();
        self.tokens.iter().try_for_each(|token| match &token.0 {
            // Simply add plain text to the result generated value
            TokenKind::Plain(s) => {
                res.push_str(s);
                Ok(())
            }
            // Try replacing variable from options map
            TokenKind::Variable(v) => match opts.get(v) {
                Some(val) => {
                    res.push_str(val);
                    Ok(())
                }
                None => Err(TransformerError::new::<Self>(
                    TransformerErrorKind::FailedToParseTempalteTransformer(
                        TemplateParserError::new(
                            super::error::TemplateParserErrorKind::FailedToResolveValueFromTemplate(
                                self.template.clone(),
                                v.clone(),
                            ),
                        ),
                    ),
                )),
            },
            // Generate random sequence of capital letter of lenght of the token and add it to
            // the result val
            TokenKind::CapitalLetterSeq(seq) => {
                res.push_str(
                    thread_rng()
                        .sample_iter(&self.upper_case_letters_set)
                        .take(seq.chars().count())
                        .collect::<String>()
                        .as_str(),
                );
                Ok(())
            }
            // Generate random sequence of lowercased letter of lenght of the token and add it to
            // the result val
            TokenKind::LowerCaseLetterSeq(seq) => {
                res.push_str(
                    thread_rng()
                        .sample_iter(&self.lower_case_letters_set)
                        .take(seq.chars().count())
                        .collect::<String>()
                        .as_str(),
                );
                Ok(())
            }
            // Generate random sequence of digits of lenght of the token and add it to
            // the result val
            TokenKind::DigitSeq(seq) => {
                res.push_str(
                    thread_rng()
                        .sample_iter(&self.digits_set)
                        .take(seq.chars().count())
                        .collect::<String>()
                        .as_str(),
                );
                Ok(())
            }
        })?;

        Ok(GeneratedValue::String(res))
    }
}
