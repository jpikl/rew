use crate::pattern::lexer::{Lexer, Token};
use crate::pattern::parse::{ParseError, Parsed};
use crate::pattern::transform::Transform;
use crate::pattern::variable::Variable;
use crate::pattern::PatternItem;

pub struct Parser {
    lexer: Lexer,
    start: usize,
    end: usize,
}

impl Parser {
    pub fn new(string: &str) -> Self {
        Self {
            lexer: Lexer::new(string),
            start: 0,
            end: 0,
        }
    }

    pub fn parse_item(&mut self) -> Result<Option<Parsed<PatternItem>>, ParseError> {
        if let Some(token) = self.fetch_token() {
            match token.value {
                Token::Raw(raw) => Ok(Some(Parsed {
                    value: PatternItem::Constant(raw),
                    start: token.start,
                    end: token.end,
                })),
                Token::ExprStart => self.parse_expression(),
                _ => Err(ParseError {
                    message: "Unexpected token",
                    start: token.start,
                    end: token.end,
                }),
            }
        } else {
            Ok(None)
        }
    }

    fn parse_expression(&mut self) -> Result<Option<Parsed<PatternItem>>, ParseError> {
        let start = self.start;
        let variable = self.parse_variable()?;
        let transforms = self.parse_transforms()?;
        let end = self.end;

        Ok(Some(Parsed {
            value: PatternItem::Expression {
                variable,
                transforms,
            },
            start,
            end,
        }))
    }

    fn parse_variable(&mut self) -> Result<Parsed<Variable>, ParseError> {
        self.parse_expression_member(Variable::parse, "Expected variable")
    }

    fn parse_transforms(&mut self) -> Result<Vec<Parsed<Transform>>, ParseError> {
        let mut transforms: Vec<Parsed<Transform>> = Vec::new();
        let mut expression_closed = false;

        while let Some(token) = self.fetch_token() {
            match token.value {
                Token::Pipe => {
                    transforms.push(self.parse_transform()?);
                }
                Token::ExprEnd => {
                    expression_closed = true;
                    break;
                }
                _ => {
                    break;
                }
            }
        }

        if expression_closed {
            Ok(transforms)
        } else {
            Err(ParseError {
                message: "Expected pipe or expression end",
                start: self.end,
                end: self.end,
            })
        }
    }

    fn parse_transform(&mut self) -> Result<Parsed<Transform>, ParseError> {
        self.parse_expression_member(Transform::parse, "Expected transformation")
    }

    fn parse_expression_member<T, F: FnOnce(&str) -> Result<T, ParseError>>(
        &mut self,
        parse: F,
        err_message: &'static str,
    ) -> Result<Parsed<T>, ParseError> {
        let position = self.end;
        let token = self.fetch_token().ok_or_else(|| ParseError {
            message: err_message,
            start: position,
            end: position,
        })?;
        if let Token::Raw(raw) = token.value {
            Ok(Parsed {
                value: parse(&raw).map_err(|mut error| {
                    error.start += position;
                    error.end += position;
                    error
                })?,
                start: token.start,
                end: token.end,
            })
        } else {
            Err(ParseError {
                message: err_message,
                start: token.start,
                end: token.end,
            })
        }
    }

    fn fetch_token(&mut self) -> Option<Parsed<Token>> {
        self.lexer.next().map(|token| {
            self.start = token.start;
            self.end = token.end;
            token
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::range::Range;
    use crate::pattern::substitution::Substitution;

    #[test]
    fn empty() {
        Parser::new("").assert_none();
    }

    #[test]
    fn constant() {
        let mut parser = Parser::new("abc");
        parser.assert_item(Parsed {
            value: PatternItem::Constant("abc".to_string()),
            start: 0,
            end: 3,
        });
        parser.assert_none();
    }

    #[test]
    fn variable() {
        let mut parser = Parser::new("{f}");
        parser.assert_item(Parsed {
            value: PatternItem::Expression {
                variable: Parsed {
                    value: Variable::Filename,
                    start: 1,
                    end: 2,
                },
                transforms: Vec::new(),
            },
            start: 0,
            end: 3,
        });
        parser.assert_none();
    }

    #[test]
    fn variable_single_transform() {
        let mut parser = Parser::new("{b|u}");
        parser.assert_item(Parsed {
            value: PatternItem::Expression {
                variable: Parsed {
                    value: Variable::Basename,
                    start: 1,
                    end: 2,
                },
                transforms: vec![Parsed {
                    value: Transform::Lowercase,
                    start: 3,
                    end: 4,
                }],
            },
            start: 0,
            end: 5,
        });
        parser.assert_none();
    }

    #[test]
    fn variable_multiple_transforms() {
        let mut parser = Parser::new("{e|t|s1-3}");
        parser.assert_item(Parsed {
            value: PatternItem::Expression {
                variable: Parsed {
                    value: Variable::Extension,
                    start: 1,
                    end: 2,
                },
                transforms: vec![
                    Parsed {
                        value: Transform::Trim,
                        start: 3,
                        end: 4,
                    },
                    Parsed {
                        value: Transform::Substring(Range {
                            offset: 0,
                            length: 3,
                        }),
                        start: 5,
                        end: 9,
                    },
                ],
            },
            start: 0,
            end: 10,
        });
        parser.assert_none();
    }

    #[test]
    fn invalid_variable_error() {
        let mut parser = Parser::new("{x}");
        parser.assert_error(ParseError {
            message: "Unknown variable",
            start: 1,
            end: 2,
        });
    }

    #[test]
    fn variable_invalid_transform_error() {
        let mut parser = Parser::new("{f|s2-1}");
        parser.assert_error(ParseError {
            message: "Range end cannot precede start",
            start: 4,
            end: 7,
        });
    }

    #[test]
    fn unexpected_token_error() {
        let mut parser = Parser::new("a}b");
        parser.assert_item(Parsed {
            value: PatternItem::Constant("a".to_string()),
            start: 0,
            end: 1,
        });
        parser.assert_error(ParseError {
            message: "Unexpected token",
            start: 1,
            end: 2,
        });
    }

    #[test]
    fn expected_variable_error() {
        let mut parser = Parser::new("{");
        parser.assert_error(ParseError {
            message: "Expected variable",
            start: 1,
            end: 1,
        });
    }

    #[test]
    fn expected_pipe_or_expr_end_error() {
        let mut parser = Parser::new("{f");
        parser.assert_error(ParseError {
            message: "Expected pipe or expression end",
            start: 2,
            end: 2,
        });
    }

    #[test]
    fn expected_transform_error() {
        let mut parser = Parser::new("{f|");
        parser.assert_error(ParseError {
            message: "Expected transformation",
            start: 3,
            end: 3,
        });
    }

    #[test]
    fn complex_input() {
        let mut parser = Parser::new("image_{c|>000}.{e|u|r'e}");
        parser.assert_item(Parsed {
            value: PatternItem::Constant("image_".to_string()),
            start: 0,
            end: 6,
        });
        parser.assert_item(Parsed {
            value: PatternItem::Expression {
                variable: Parsed {
                    value: Variable::LocalCounter,
                    start: 7,
                    end: 8,
                },
                transforms: vec![Parsed {
                    value: Transform::LeftPad(vec!['0', '0', '0']),
                    start: 9,
                    end: 13,
                }],
            },
            start: 6,
            end: 14,
        });
        parser.assert_item(Parsed {
            value: PatternItem::Constant(".".to_string()),
            start: 14,
            end: 15,
        });
        parser.assert_item(Parsed {
            value: PatternItem::Expression {
                variable: Parsed {
                    value: Variable::Extension,
                    start: 16,
                    end: 17,
                },
                transforms: vec![
                    Parsed {
                        value: Transform::Lowercase,
                        start: 18,
                        end: 19,
                    },
                    Parsed {
                        value: Transform::ReplaceFirst(Substitution {
                            value: 'e'.to_string(),
                            replacement: String::new(),
                        }),
                        start: 20,
                        end: 23,
                    },
                ],
            },
            start: 15,
            end: 24,
        });
        parser.assert_none();
    }

    impl Parser {
        fn assert_none(&mut self) {
            assert_eq!(self.parse_item(), Ok(None));
        }

        fn assert_item(&mut self, item: Parsed<PatternItem>) {
            assert_eq!(self.parse_item(), Ok(Some(item)));
        }

        fn assert_error(&mut self, error: ParseError) {
            assert_eq!(self.parse_item(), Err(error));
        }
    }
}
