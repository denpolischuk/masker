use std::borrow::Borrow;

use super::{GeneratedValue, TransformerError};
use crate::masker::{
    error::{ConfigParseError, ConfigParseErrorKind},
    transformer::{Options, Transformer},
};

#[derive(Debug, PartialEq)]
enum TokenKind {
    Variable(String),
    CapitalLetterSeq(String),
    LowerCaseLetterSeq(String),
    DigitsSeq(String),
}

#[derive(Debug, PartialEq)]
struct Token {
    start_p: usize,
    end_p: usize,
    placeholder: TokenKind,
}

// Enum describes possible state of parser state machine.
// Each enum tumple contains a PREVIOUS ITERATION char and char index of the string
enum VariableParserState {
    Plain(usize, char),         // just string reading
    VarEntry(usize, char),      // '#' char detected
    VarBlockStart(usize, char), // if '(' follows after '#'
    VarTokenRead(usize, char),  // whatever comes after #( and is alphanumerical or underscore
    SeqBlockStart(usize, char), // if '{' follows after '#'
    SeqTokenRead(usize, char), // whatever comes after #( and is part of known rand sequence charset
}

pub struct TemplateTransformer {
    template: String,
    tokens: Vec<Token>,
}

impl TemplateTransformer {
    pub fn new_from_yaml(yaml: &serde_yaml::Value) -> Result<Self, ConfigParseError> {
        let field = "template";
        match yaml[field].as_str() {
            Some(t) => {
                let tokens = Self::parse_variables(t.to_string().borrow()).map_err(|e| {
                    ConfigParseError {
                        field: field.to_string(),
                        kind: ConfigParseErrorKind::FailedToCreateTransformerFromConfig(e),
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

    // State machine function that parses random symbol sequences and creates tokens from them
    fn parse_random_sequences(template: &String) -> Result<Vec<Token>, TransformerError> {
        todo!()
    }

    // State machine function that parses the template string, detects variable tokens and creates
    // a tokens vector out of them
    fn parse_variables(template: &String) -> Result<Vec<Token>, TransformerError> {
        let mut state = VariableParserState::Plain(0, '!');
        let mut tokens: Vec<Token> = vec![];

        let map_res: Result<Vec<()>, TransformerError> = template
            .char_indices()
            .map(|(ind, ch)| -> Result<(), TransformerError> {
                state = match state {
                    VariableParserState::Plain(_prev_ind, _prev_charr) => {
                        if ch == '%' {
                            tokens.push(Token {
                                start_p: ind,
                                end_p: 0,
                                placeholder: TokenKind::Variable(String::new()),
                            });
                            VariableParserState::VarEntry(ind, ch)
                        } else {
                            VariableParserState::Plain(ind, ch)
                        }
                    }
                    VariableParserState::VarEntry(_prev_ind, _prev_char) => {
                        if ch == '(' {
                            VariableParserState::VarBlockStart(ind, ch)
                        } else {
                            tokens.pop();
                            VariableParserState::Plain(ind, ch)
                        }
                    }
                    VariableParserState::VarBlockStart(_prev_ind, _prev_char) => {
                        if ch.is_ascii_alphanumeric() || ch == '_' {
                            let mut token =
                                match tokens.pop() {
                                    Some(t) => t,
                                    None => return Err(TransformerError::new::<Self>(
                                        super::error::TransformerErrorKind::FailedToParseTemplate(
                                            template.clone(),
                                            ind,
                                        ),
                                    )),
                                };
                            token.placeholder = TokenKind::Variable(String::from(ch));
                            tokens.push(token);
                            VariableParserState::VarTokenRead(ind, ch)
                        } else {
                            tokens.pop();
                            VariableParserState::Plain(ind, ch)
                        }
                    }
                    VariableParserState::VarTokenRead(_prev_ind, _prev_char) => {
                        match tokens.pop() {
                            Some(mut token) => match token.placeholder {
                                TokenKind::Variable(placeholder) => {
                                    if ch.is_ascii_alphanumeric() || ch == '_' {
                                        token.placeholder =
                                            TokenKind::Variable(format!("{}{}", placeholder, ch));
                                        tokens.push(token);
                                        VariableParserState::VarTokenRead(ind, ch)
                                    } else if ch == ')' {
                                        token.end_p = ind;
                                        token.placeholder =
                                            TokenKind::Variable(placeholder.clone());
                                        tokens.push(token);
                                        VariableParserState::Plain(ind, ch)
                                    } else {
                                        return Err(TransformerError::new::<Self>(
                                            super::error::TransformerErrorKind::UnexpectedToken(
                                                template.clone(),
                                                ind,
                                                ch,
                                            ),
                                        ));
                                    }
                                }
                                _ => {
                                    return Err(TransformerError::new::<Self>(
                                        super::error::TransformerErrorKind::FailedToParseTemplate(
                                            template.clone(),
                                            ind,
                                        ),
                                    ))
                                }
                            },
                            None => {
                                return Err(TransformerError::new::<Self>(
                                    super::error::TransformerErrorKind::FailedToParseTemplate(
                                        template.clone(),
                                        ind,
                                    ),
                                ))
                            }
                        }
                    }
                    VariableParserState::SeqBlockStart(_, _) => todo!(),
                    VariableParserState::SeqTokenRead(_, _) => todo!(),
                };
                Ok(())
            })
            .collect();

        map_res?;

        match state {
            VariableParserState::Plain(_, _) => Ok(tokens),
            _ => Err(TransformerError::new::<Self>(
                super::error::TransformerErrorKind::FailedToParseTemplate(template.to_string(), 0),
            )),
        }
    }
}

impl Transformer for TemplateTransformer {
    fn generate(&self, opts: &Options) -> Result<GeneratedValue, TransformerError> {
        let res = self.template.replace("{id}", opts.pk.to_string().as_str());

        Ok(GeneratedValue::String(res))
    }
}

#[cfg(test)]
mod tests {
    use crate::masker::transformer::error::TransformerErrorKind;

    use super::{TemplateTransformer, Token, TokenKind};

    #[test]
    fn it_parses_variables() {
        let template = String::from("Company #%(id)");

        let exp = Token {
            start_p: 9,
            end_p: 13,
            placeholder: TokenKind::Variable(String::from("id")),
        };

        let res = TemplateTransformer::parse_variables(&template)
            .unwrap()
            .pop()
            .unwrap();

        assert_eq!(exp, res);
    }

    #[test]
    fn it_panics_on_unclosed_var_token() {
        let template = String::from("Company #%(id");
        let err = TemplateTransformer::parse_variables(&template).unwrap_err();

        assert_eq!(
            err.kind,
            TransformerErrorKind::FailedToParseTemplate(template, 0)
        );
    }

    #[test]
    fn returns_no_tokens_when_nothing_to_parse() {
        let template = String::from("Company (id)");
        let res = TemplateTransformer::parse_variables(&template).unwrap();

        assert!(res.is_empty());
    }

    #[test]
    fn parses_random_sequences() {
        let tpl = String::from("Company %{Llllll}-%{dd}");

        let r = TemplateTransformer::parse_random_sequences(&tpl).unwrap();
    }
}
