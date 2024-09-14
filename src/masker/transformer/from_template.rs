use std::borrow::Borrow;

use crate::masker::{
    error::{ConfigParseError, ConfigParseErrorKind},
    transformer::{Options, Transformer},
};

#[derive(Debug, PartialEq)]
struct Token {
    start_p: usize,
    end_p: usize,
    placeholder: String,
}

use super::{tranformer::GeneratedValue, TransformerError};
pub struct TemplateTransformer {
    template: String,
    tokens: Vec<Token>,
}

impl TemplateTransformer {
    pub fn new(template: String, tokens: Vec<Token>) -> TemplateTransformer {
        Self {
            template: template.clone(),
            tokens,
        }
    }

    pub fn new_from_yaml(yaml: &serde_yaml::Value) -> Result<Self, ConfigParseError> {
        let field = "template";
        match yaml[field].as_str() {
            Some(t) => {
                let tokens = Self::parse_var_tokens(t.to_string().borrow()).map_err(|e| {
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

    pub fn parse_var_tokens(template: &String) -> Result<Vec<Token>, TransformerError> {
        let mut tokens: Vec<Token> = vec![];
        let mut stack: Vec<char> = vec![];
        let mut expect_var_next = false;
        let mut prev_ind: usize = 0;
        let map_res: Result<Vec<()>, TransformerError> = template
            .char_indices()
            .map(|(ind, ch)| -> Result<(), TransformerError> {
                if ch == '%' && !expect_var_next {
                    expect_var_next = true;
                    prev_ind = ind;
                    return Ok(());
                }
                if ch == '(' && expect_var_next {
                    stack.push(ch);
                    tokens.push(Token {
                        start_p: prev_ind,
                        end_p: 0,
                        placeholder: String::new(),
                    })
                }
                if ch == ')' && stack.ends_with(&['(']) {
                    stack.pop();
                    let mut token = match tokens.pop() {
                        Some(t) => t,
                        None => {
                            return Err(TransformerError::new::<Self>(
                                super::error::TransformerErrorKind::FailedToParseTemplate(
                                    template.clone(),
                                    ind,
                                ),
                            ))
                        }
                    };

                    token.end_p = ind + 1;
                    token.placeholder = template[token.start_p + 2..ind].to_string();
                    tokens.push(token);
                }
                prev_ind = ind;
                expect_var_next = false;
                Ok(())
            })
            .collect();
        map_res?;
        if !stack.is_empty() {
            return Err(TransformerError::new::<Self>(
                super::error::TransformerErrorKind::FailedToParseTemplate(template.to_string(), 0),
            ));
        }
        Ok(tokens)
    }
}

impl Default for TemplateTransformer {
    fn default() -> Self {
        Self::new(String::from("example.com"), vec![])
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

    use super::{TemplateTransformer, Token};

    #[test]
    fn it_parses_variables() {
        let template = String::from("Company #%(id)");

        let exp = Token {
            start_p: 9,
            end_p: 14,
            placeholder: String::from("id"),
        };

        let res = TemplateTransformer::parse_var_tokens(&template)
            .unwrap()
            .pop()
            .unwrap();

        assert_eq!(res, exp);

        let st = format!(
            "{}{}{}",
            &template[..res.start_p],
            "0",
            &template[res.end_p..]
        );
        assert_eq!(st, template.replace("%(id)", "0"))
    }

    #[test]
    fn it_panics_on_unclosed_var_token() {
        let template = String::from("Company #%(id");
        let err = TemplateTransformer::parse_var_tokens(&template).unwrap_err();

        assert_eq!(
            err.kind,
            TransformerErrorKind::FailedToParseTemplate(template, 0)
        );
    }

    #[test]
    fn returns_no_tokens_when_nothing_to_parse() {
        let template = String::from("Company (id)");
        let res = TemplateTransformer::parse_var_tokens(&template).unwrap();

        assert!(res.is_empty());
    }
}
