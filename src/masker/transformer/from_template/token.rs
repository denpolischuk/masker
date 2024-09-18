use super::error::{TemplateParserError, TemplateParserErrorKind};

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Plain(String),
    Variable(String),
    CapitalLetterSeq(String),
    LowerCaseLetterSeq(String),
    DigitSeq(String),
}

#[derive(Debug, PartialEq)]
pub struct Token(pub TokenKind);

// Enum describes possible state of parser state machine.
// Each enum tumple contains a PREVIOUS ITERATION char and char index of the string
enum VariableParserState {
    Plain,         // just string reading
    TokenEntry,    // '#' char detected
    VarBlockStart, // if '(' follows after '#'
    VarTokenRead,  // whatever comes after #( and is alphanumerical or underscore
    SeqBlockStart, // if '{' follows after '#'
    SeqTokenRead,  // whatever comes after #( and is part of known rand sequence charset
}

impl Token {
    // State machine function that parses the template string, detects variable tokens or random
    // sequence and creates a tokens vector out of them
    pub fn parse_tokens_from_template(
        template: &String,
    ) -> Result<Vec<Token>, TemplateParserError> {
        let mut state = VariableParserState::Plain;
        let mut tokens: Vec<Token> = vec![];

        let map_res: Result<Vec<()>, TemplateParserError> = template
            .char_indices()
            .map(|(ind, ch)| -> Result<(), TemplateParserError> {
                // Updates machine state every char iteration based on char
                // it meets
                state = match state {
                    // Plain means it just iterates over sequence of string value that shouldn't be
                    // parsed/replaced
                    VariableParserState::Plain => {
                        // If it finds % token then it signalizes that there might be either
                        // variable or random sequnce in front
                        if ch == '%' {
                            tokens.push(Token(TokenKind::Variable(String::new())));
                            VariableParserState::TokenEntry
                        } else {
                            let mut token = match tokens.pop() {
                                Some(token) => token,
                                None => Token(TokenKind::Plain(String::new())),
                            };
                            let token = match token.0 {
                                TokenKind::Plain(placeholder) => {
                                    token.0 = TokenKind::Plain(format!("{}{}", placeholder, ch));
                                    token
                                }
                                _ => {
                                    tokens.push(token);
                                    Token(TokenKind::Plain(String::from(ch)))
                                }
                            };
                            tokens.push(token);
                            VariableParserState::Plain
                        }
                    }
                    VariableParserState::TokenEntry => match ch {
                        // Variable token starts with with ( after % indicator
                        '(' => VariableParserState::VarBlockStart,
                        // { is an indicator of a sequence of randomized chars
                        '{' => VariableParserState::SeqBlockStart,
                        _ => {
                            tokens.pop();
                            tokens.push(Token(TokenKind::Plain(String::from(ch))));
                            VariableParserState::Plain
                        }
                    },
                    // Handles only variable tokens
                    VariableParserState::VarBlockStart => {
                        // variable name can only be letter, number of _
                        if ch.is_ascii_alphanumeric() || ch == '_' {
                            let mut token = match tokens.pop() {
                                Some(t) => t,
                                None => {
                                    return Err(TemplateParserError::new(
                                        TemplateParserErrorKind::FailedToParseTemplate(
                                            template.clone(),
                                            ind,
                                        ),
                                    ))
                                }
                            };
                            token.0 = TokenKind::Variable(String::from(ch));
                            tokens.push(token);
                            VariableParserState::VarTokenRead
                        } else {
                            tokens.pop();
                            VariableParserState::Plain
                        }
                    }
                    // Keeps reading the name of variable
                    VariableParserState::VarTokenRead => {
                        match tokens.pop() {
                            Some(mut token) => match token.0 {
                                TokenKind::Variable(placeholder) => {
                                    // Keep reading if letter, digit or _ has been met
                                    if ch.is_ascii_alphanumeric() || ch == '_' {
                                        token.0 =
                                            TokenKind::Variable(format!("{}{}", placeholder, ch));
                                        tokens.push(token);
                                        VariableParserState::VarTokenRead
                                    // If ) char has been read, then it means that variable token
                                    // block is closing and state should return back to plain
                                    } else if ch == ')' {
                                        token.0 = TokenKind::Variable(placeholder.clone());
                                        tokens.push(token);
                                        VariableParserState::Plain
                                    // Everything else should cause an error
                                    } else {
                                        return Err(TemplateParserError::new(
                                            TemplateParserErrorKind::UnexpectedToken(
                                                template.clone(),
                                                ind,
                                                ch,
                                            ),
                                        ));
                                    }
                                }
                                _ => {
                                    return Err(TemplateParserError::new(
                                        TemplateParserErrorKind::FailedToParseTemplate(
                                            template.clone(),
                                            ind,
                                        ),
                                    ))
                                }
                            },
                            None => {
                                return Err(TemplateParserError::new(
                                    TemplateParserErrorKind::FailedToParseTemplate(
                                        template.clone(),
                                        ind,
                                    ),
                                ))
                            }
                        }
                    }
                    // Next two hands handle sequence of random char tokens
                    VariableParserState::SeqBlockStart => match tokens.pop() {
                        Some(mut token) => match ch {
                            // L stands for random capital letter character
                            'L' => {
                                token.0 = TokenKind::CapitalLetterSeq(String::from(ch));
                                tokens.push(token);
                                VariableParserState::SeqTokenRead
                            }
                            // l - is a lowercase letter character
                            'l' => {
                                token.0 = TokenKind::LowerCaseLetterSeq(String::from(ch));
                                tokens.push(token);
                                VariableParserState::SeqTokenRead
                            }
                            // d - stands for random digit 0-9
                            'd' => {
                                token.0 = TokenKind::DigitSeq(String::from(ch));
                                tokens.push(token);
                                VariableParserState::SeqTokenRead
                            }
                            // Everything else is unrecognized and thus should throw an error
                            _ => {
                                return Err(TemplateParserError::new(
                                    TemplateParserErrorKind::UnrecognizedSequenceSymbol(
                                        template.clone(),
                                        ind,
                                        ch,
                                    ),
                                ))
                            }
                        },
                        None => {
                            return Err(TemplateParserError::new(
                                TemplateParserErrorKind::FailedToParseTemplate(
                                    template.clone(),
                                    ind,
                                ),
                            ))
                        }
                    },
                    VariableParserState::SeqTokenRead => {
                        match tokens.pop() {
                            Some(mut token) => match ch {
                                // If L is met and prev state is also CapitalLetterSeq, then keep
                                // reading into the same Token
                                'L' => match token.0 {
                                    TokenKind::CapitalLetterSeq(placeholder) => {
                                        token.0 = TokenKind::CapitalLetterSeq(format!(
                                            "{placeholder}{ch}"
                                        ));
                                        tokens.push(token);
                                        VariableParserState::SeqTokenRead
                                    }
                                    // If prev state is something different, then finalize the token
                                    // and create a new one for the capital letter sequence
                                    _ => {
                                        tokens.push(token);
                                        let token =
                                            Token(TokenKind::CapitalLetterSeq(String::from(ch)));
                                        tokens.push(token);
                                        VariableParserState::SeqTokenRead
                                    }
                                },
                                // Same logic as for L, but for lowercased letter sequence
                                'l' => match token.0 {
                                    TokenKind::LowerCaseLetterSeq(placeholder) => {
                                        token.0 = TokenKind::LowerCaseLetterSeq(format!(
                                            "{placeholder}{ch}"
                                        ));
                                        tokens.push(token);
                                        VariableParserState::SeqTokenRead
                                    }
                                    _ => {
                                        tokens.push(token);
                                        let token =
                                            Token(TokenKind::LowerCaseLetterSeq(String::from(ch)));
                                        tokens.push(token);
                                        VariableParserState::SeqTokenRead
                                    }
                                },
                                // Same logic as for L, but for digits sequence
                                'd' => match token.0 {
                                    TokenKind::DigitSeq(placeholder) => {
                                        token.0 = TokenKind::DigitSeq(format!("{placeholder}{ch}"));
                                        tokens.push(token);
                                        VariableParserState::SeqTokenRead
                                    }
                                    _ => {
                                        tokens.push(token);
                                        let token = Token(TokenKind::DigitSeq(String::from(ch)));
                                        tokens.push(token);
                                        VariableParserState::SeqTokenRead
                                    }
                                },
                                // If } is met, then should finalize current token
                                '}' => {
                                    tokens.push(token);
                                    VariableParserState::Plain
                                }
                                _ => {
                                    return Err(TemplateParserError::new(
                                        TemplateParserErrorKind::UnrecognizedSequenceSymbol(
                                            template.clone(),
                                            ind,
                                            ch,
                                        ),
                                    ))
                                }
                            },
                            None => {
                                return Err(TemplateParserError::new(
                                    TemplateParserErrorKind::FailedToParseTemplate(
                                        template.clone(),
                                        ind,
                                    ),
                                ))
                            }
                        }
                    }
                };
                Ok(())
            })
            .collect();

        map_res?;

        // State machine should always endup in Plain state.
        // If state is something else, then it means that template is incorrect.
        // Specifically this match covers sitautions like this template "Company #%{dd} %{Llll",
        // where last sequence is not ended with closing character }
        match state {
            VariableParserState::Plain => Ok(tokens),
            _ => Err(TemplateParserError::new(
                TemplateParserErrorKind::FailedToParseTemplate(template.to_string(), 0),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::masker::transformer::from_template::error::{
        TemplateParserError, TemplateParserErrorKind,
    };

    use super::{Token, TokenKind};

    #[test]
    fn it_parses_variables() {
        let template = String::from("Company #%(id) llc");

        let expected = vec![
            Token(TokenKind::Plain(String::from("Company #"))),
            Token(TokenKind::Variable(String::from("id"))),
            Token(TokenKind::Plain(String::from(" llc"))),
        ];

        let res = Token::parse_tokens_from_template(&template).unwrap();

        assert_eq!(expected, res);
    }

    #[test]
    fn it_panics_on_unclosed_var_token() {
        let template = String::from("Company #%(id");
        let err = Token::parse_tokens_from_template(&template).unwrap_err();

        assert_eq!(
            err.kind,
            TemplateParserErrorKind::FailedToParseTemplate(template, 0)
        );
    }

    #[test]
    fn returns_only_plain_token_when_no_var_or_seq_found() {
        let template = String::from("Company (id)");
        let mut res = Token::parse_tokens_from_template(&template).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(
            res.pop().unwrap().0,
            TokenKind::Plain(String::from("Company (id)"))
        )
    }

    #[test]
    fn returns_only_plain_token_when_token_trigger_char_is_escaped() {
        let template = String::from("Company %%(id)");
        let mut res = Token::parse_tokens_from_template(&template).unwrap();

        assert_eq!(res.len(), 2);
        assert_eq!(
            res.pop().unwrap().0,
            TokenKind::Plain(String::from("%(id)"))
        );
        assert_eq!(
            res.pop().unwrap().0,
            TokenKind::Plain(String::from("Company "))
        )
    }

    #[test]
    fn parses_random_sequences() {
        let tpl = String::from("C %{Llllll}-%{dd}");
        let exp = vec![
            Token(TokenKind::Plain(String::from("C "))),
            Token(TokenKind::CapitalLetterSeq(String::from("L"))),
            Token(TokenKind::LowerCaseLetterSeq(String::from("lllll"))),
            Token(TokenKind::Plain(String::from("-"))),
            Token(TokenKind::DigitSeq(String::from("dd"))),
        ];

        let r = Token::parse_tokens_from_template(&tpl).unwrap();
        assert_eq!(r.len(), 5);
        assert_eq!(r, exp);
    }

    #[test]
    fn fails_to_parse_sequence_when_unexpected_token_found() {
        let tpl = String::from("C %{Lllxll}-%{dd}");
        let r = Token::parse_tokens_from_template(&tpl).unwrap_err();
        assert_eq!(
            r,
            TemplateParserError::new(TemplateParserErrorKind::UnrecognizedSequenceSymbol(
                tpl, 7, 'x'
            ))
        );
    }
}
