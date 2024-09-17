use super::error::{TemplateParserError, TemplateParserErrorKind};

#[derive(Debug, PartialEq)]
enum TokenKind {
    Variable(String),
    CapitalLetterSeq(String),
    LowerCaseLetterSeq(String),
    DigitSeq(String),
}

#[derive(Debug, PartialEq)]
pub struct Token {
    start_p: usize,
    end_p: usize,
    placeholder: TokenKind,
}

// Enum describes possible state of parser state machine.
// Each enum tumple contains a PREVIOUS ITERATION char and char index of the string
enum VariableParserState {
    Plain(usize, char),         // just string reading
    TokenEntry(usize, char),    // '#' char detected
    VarBlockStart(usize, char), // if '(' follows after '#'
    VarTokenRead(usize, char),  // whatever comes after #( and is alphanumerical or underscore
    SeqBlockStart(usize, char), // if '{' follows after '#'
    SeqTokenRead(usize, char), // whatever comes after #( and is part of known rand sequence charset
}

impl Token {
    // State machine function that parses the template string, detects variable tokens or random
    // sequence and creates a tokens vector out of them
    pub fn parse_tokens_from_template(
        template: &String,
    ) -> Result<Vec<Token>, TemplateParserError> {
        let mut state = VariableParserState::Plain(0, '!');
        let mut tokens: Vec<Token> = vec![];

        let map_res: Result<Vec<()>, TemplateParserError> = template
            .char_indices()
            .map(|(ind, ch)| -> Result<(), TemplateParserError> {
                // Updates machine state every char iteration based on char
                // it meets
                state = match state {
                    // Plain means it just iterates over sequence of string value that shouldn't be
                    // parsed/replaced
                    VariableParserState::Plain(_prev_ind, _prev_charr) => {
                        // If it finds % token then it signalizes that there might be either
                        // variable or random sequnce in front
                        if ch == '%' {
                            tokens.push(Token {
                                start_p: ind,
                                end_p: 0,
                                placeholder: TokenKind::Variable(String::new()),
                            });
                            VariableParserState::TokenEntry(ind, ch)
                        } else {
                            VariableParserState::Plain(ind, ch)
                        }
                    }
                    VariableParserState::TokenEntry(_prev_ind, _prev_char) => match ch {
                        // Variable token starts with with ( after % indicator
                        '(' => VariableParserState::VarBlockStart(ind, ch),
                        // { is an indicator of a sequence of randomized chars
                        '{' => VariableParserState::SeqBlockStart(ind, ch),
                        _ => {
                            tokens.pop();
                            VariableParserState::Plain(ind, ch)
                        }
                    },
                    // Handles only variable tokens
                    VariableParserState::VarBlockStart(_prev_ind, _prev_char) => {
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
                            token.placeholder = TokenKind::Variable(String::from(ch));
                            tokens.push(token);
                            VariableParserState::VarTokenRead(ind, ch)
                        } else {
                            tokens.pop();
                            VariableParserState::Plain(ind, ch)
                        }
                    }
                    // Keeps reading the name of variable
                    VariableParserState::VarTokenRead(_prev_ind, _prev_char) => {
                        match tokens.pop() {
                            Some(mut token) => match token.placeholder {
                                TokenKind::Variable(placeholder) => {
                                    // Keep reading if letter, digit or _ has been met
                                    if ch.is_ascii_alphanumeric() || ch == '_' {
                                        token.placeholder =
                                            TokenKind::Variable(format!("{}{}", placeholder, ch));
                                        tokens.push(token);
                                        VariableParserState::VarTokenRead(ind, ch)
                                    // If ) char has been read, then it means that variable token
                                    // block is closing and state should return back to plain
                                    } else if ch == ')' {
                                        token.end_p = ind;
                                        token.placeholder =
                                            TokenKind::Variable(placeholder.clone());
                                        tokens.push(token);
                                        VariableParserState::Plain(ind, ch)
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
                    VariableParserState::SeqBlockStart(_prev_ind, _prev_char) => match tokens.pop()
                    {
                        Some(mut token) => match ch {
                            // L stands for random capital letter character
                            'L' => {
                                token.placeholder = TokenKind::CapitalLetterSeq(String::from(ch));
                                tokens.push(token);
                                VariableParserState::SeqTokenRead(ind, ch)
                            }
                            // l - is a lowercase letter character
                            'l' => {
                                token.placeholder = TokenKind::LowerCaseLetterSeq(String::from(ch));
                                tokens.push(token);
                                VariableParserState::SeqTokenRead(ind, ch)
                            }
                            // d - stands for random digit 0-9
                            'd' => {
                                token.placeholder = TokenKind::DigitSeq(String::from(ch));
                                tokens.push(token);
                                VariableParserState::SeqTokenRead(ind, ch)
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
                    VariableParserState::SeqTokenRead(prev_ind, _prev_char) => match tokens.pop() {
                        Some(mut token) => match ch {
                            // If L is met and prev state is also CapitalLetterSeq, then keep
                            // reading into the same Token
                            'L' => match token.placeholder {
                                TokenKind::CapitalLetterSeq(placeholder) => {
                                    token.placeholder =
                                        TokenKind::CapitalLetterSeq(format!("{placeholder}{ch}"));
                                    tokens.push(token);
                                    VariableParserState::SeqTokenRead(ind, ch)
                                }
                                // If prev state is something different, then finalize the token
                                // and create a new one for the capital letter sequence
                                _ => {
                                    token.end_p = prev_ind;
                                    tokens.push(token);
                                    let token = Token {
                                        start_p: ind,
                                        end_p: ind,
                                        placeholder: TokenKind::CapitalLetterSeq(String::from(ch)),
                                    };
                                    tokens.push(token);
                                    VariableParserState::SeqTokenRead(ind, ch)
                                }
                            },
                            // Same logic as for L, but for lowercased letter sequence
                            'l' => match token.placeholder {
                                TokenKind::LowerCaseLetterSeq(placeholder) => {
                                    token.placeholder =
                                        TokenKind::LowerCaseLetterSeq(format!("{placeholder}{ch}"));
                                    tokens.push(token);
                                    VariableParserState::SeqTokenRead(ind, ch)
                                }
                                _ => {
                                    token.end_p = prev_ind;
                                    tokens.push(token);
                                    let token = Token {
                                        start_p: ind,
                                        end_p: ind,
                                        placeholder: TokenKind::LowerCaseLetterSeq(String::from(
                                            ch,
                                        )),
                                    };
                                    tokens.push(token);
                                    VariableParserState::SeqTokenRead(ind, ch)
                                }
                            },
                            // Same logic as for L, but for digits sequence
                            'd' => match token.placeholder {
                                TokenKind::DigitSeq(placeholder) => {
                                    token.placeholder =
                                        TokenKind::DigitSeq(format!("{placeholder}{ch}"));
                                    tokens.push(token);
                                    VariableParserState::SeqTokenRead(ind, ch)
                                }
                                _ => {
                                    token.end_p = prev_ind;
                                    tokens.push(token);
                                    let token = Token {
                                        start_p: ind,
                                        end_p: ind,
                                        placeholder: TokenKind::DigitSeq(String::from(ch)),
                                    };
                                    tokens.push(token);
                                    VariableParserState::SeqTokenRead(ind, ch)
                                }
                            },
                            // If } is met, then should finalize current token
                            '}' => {
                                token.end_p = ind;
                                tokens.push(token);
                                VariableParserState::Plain(ind, ch)
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
                    },
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
            VariableParserState::Plain(_, _) => Ok(tokens),
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
        let template = String::from("Company #%(id)");

        let exp = Token {
            start_p: 9,
            end_p: 13,
            placeholder: TokenKind::Variable(String::from("id")),
        };

        let res = Token::parse_tokens_from_template(&template)
            .unwrap()
            .pop()
            .unwrap();

        assert_eq!(exp, res);
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
    fn returns_no_tokens_when_nothing_to_parse() {
        let template = String::from("Company (id)");
        let res = Token::parse_tokens_from_template(&template).unwrap();

        assert!(res.is_empty());
    }

    #[test]
    fn parses_random_sequences() {
        let tpl = String::from("C %{Llllll}-%{dd}");
        let exp = vec![
            Token {
                start_p: 2,
                end_p: 4,
                placeholder: TokenKind::CapitalLetterSeq(String::from("L")),
            },
            Token {
                start_p: 5,
                end_p: 10,
                placeholder: TokenKind::LowerCaseLetterSeq(String::from("lllll")),
            },
            Token {
                start_p: 12,
                end_p: 16,
                placeholder: TokenKind::DigitSeq(String::from("dd")),
            },
        ];

        let r = Token::parse_tokens_from_template(&tpl).unwrap();
        assert_eq!(r.len(), 3);
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
