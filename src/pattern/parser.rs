use crate::pattern::char::Char;
use crate::pattern::filter::Filter;
use crate::pattern::lexer::{Lexer, Token};
use crate::pattern::parse::{ParseError, ParseErrorKind, ParseResult, Parsed};
use crate::pattern::reader::Reader;
use crate::pattern::variable::Variable;

#[derive(Debug, PartialEq)]
pub enum PatternItem {
    Constant(String),
    Expression {
        variable: Parsed<Variable>,
        filters: Vec<Parsed<Filter>>,
    },
}

pub struct Parser {
    lexer: Lexer,
    token: Option<Parsed<Token>>,
}

impl From<&str> for Parser {
    fn from(string: &str) -> Self {
        Self::new(Lexer::from(string))
    }
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self { lexer, token: None }
    }

    pub fn parse_items(&mut self) -> ParseResult<Vec<Parsed<PatternItem>>> {
        let mut items = Vec::new();

        while let Some(item) = self.parse_item()? {
            items.push(item);
        }

        Ok(items)
    }

    fn parse_item(&mut self) -> ParseResult<Option<Parsed<PatternItem>>> {
        if let Some(token) = self.fetch_token()? {
            match &token.value {
                Token::Raw(raw) => Ok(Some(Parsed {
                    value: PatternItem::Constant(Char::join(raw)),
                    start: token.start,
                    end: token.end,
                })),
                Token::ExprStart => {
                    let start = token.start;
                    let end = token.end;
                    let expression = self.parse_expression()?;

                    if let Some(Token::ExprEnd) = self.token_value() {
                        Ok(expression)
                    } else {
                        Err(ParseError {
                            kind: ParseErrorKind::UnmatchedExprStart,
                            start,
                            end,
                        })
                    }
                }
                Token::ExprEnd => Err(ParseError {
                    kind: ParseErrorKind::UnmatchedExprEnd,
                    start: token.start,
                    end: token.end,
                }),
                Token::Pipe => Err(ParseError {
                    kind: ParseErrorKind::PipeOutsideExpr,
                    start: token.start,
                    end: token.end,
                }),
            }
        } else {
            Ok(None)
        }
    }

    fn parse_expression(&mut self) -> ParseResult<Option<Parsed<PatternItem>>> {
        let start = self.token_start();
        let variable = self.parse_variable()?;
        let filters = self.parse_filters()?;
        let end = self.token_end();

        Ok(Some(Parsed {
            value: PatternItem::Expression { variable, filters },
            start,
            end,
        }))
    }

    fn parse_variable(&mut self) -> ParseResult<Parsed<Variable>> {
        self.parse_expression_member(Variable::parse, ParseErrorKind::ExpectedVariable)
    }

    fn parse_filters(&mut self) -> ParseResult<Vec<Parsed<Filter>>> {
        let mut filters: Vec<Parsed<Filter>> = Vec::new();

        while let Some(token) = self.fetch_token()? {
            match token.value {
                Token::Pipe => {
                    filters.push(self.parse_filter()?);
                }
                Token::ExprStart => {
                    return Err(ParseError {
                        kind: ParseErrorKind::ExprStartInsideExpr,
                        start: token.start,
                        end: token.end,
                    })
                }
                Token::ExprEnd => {
                    break;
                }
                _ => {
                    panic!("Unexpected token {:?}", token); // Raw or anything else should never appear here!
                }
            }
        }

        Ok(filters)
    }

    fn parse_filter(&mut self) -> ParseResult<Parsed<Filter>> {
        self.parse_expression_member(Filter::parse, ParseErrorKind::ExpectedFilter)
    }

    fn parse_expression_member<T, F: FnOnce(&mut Reader) -> ParseResult<T>>(
        &mut self,
        parse: F,
        error_kind: ParseErrorKind,
    ) -> ParseResult<Parsed<T>> {
        let position = self.token_end();
        let token = self.fetch_token()?.ok_or_else(|| ParseError {
            kind: error_kind.clone(),
            start: position,
            end: position,
        })?;
        if let Token::Raw(raw) = &token.value {
            let mut reader = Reader::new(raw.clone());
            let value = parse(&mut reader).map_err(|mut error| {
                error.start += position;
                error.end += position;
                error
            })?;
            if let Some(char) = reader.peek() {
                // There should be no remaining characters
                Err(ParseError {
                    kind: ParseErrorKind::ExpectedPipeOrExprEnd,
                    start: position + reader.position(),
                    end: position + reader.position() + char.len(),
                })
            } else {
                Ok(Parsed {
                    value,
                    start: token.start,
                    end: token.end,
                })
            }
        } else {
            Err(ParseError {
                kind: error_kind,
                start: token.start,
                end: token.end,
            })
        }
    }

    fn fetch_token(&mut self) -> ParseResult<Option<&Parsed<Token>>> {
        self.token = self.lexer.read_token()?;
        Ok(self.token.as_ref())
    }

    fn token_value(&self) -> Option<&Token> {
        self.token.as_ref().map(|token| &token.value)
    }

    fn token_start(&self) -> usize {
        self.token.as_ref().map_or(0, |token| token.start)
    }

    fn token_end(&self) -> usize {
        self.token.as_ref().map_or(0, |token| token.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::range::Range;
    use crate::pattern::substitution::Substitution;

    #[test]
    fn empty() {
        assert_eq!(Parser::from("").parse_items(), Ok(Vec::new()));
    }

    #[test]
    fn constant() {
        assert_eq!(
            Parser::from("a").parse_items(),
            Ok(vec![Parsed {
                value: PatternItem::Constant("a".to_string()),
                start: 0,
                end: 1,
            }])
        );
    }

    #[test]
    fn expected_variable_but_end_error() {
        assert_eq!(
            Parser::from("{").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedVariable,
                start: 1,
                end: 1,
            })
        );
    }

    #[test]
    fn expected_variable_but_pipe_error() {
        assert_eq!(
            Parser::from("{|").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedVariable,
                start: 1,
                end: 2,
            })
        );
    }

    #[test]
    fn pipe_outside_expr_error() {
        assert_eq!(
            Parser::from("|").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::PipeOutsideExpr,
                start: 0,
                end: 1,
            })
        );
    }

    #[test]
    fn expected_variable_but_expr_end_error() {
        assert_eq!(
            Parser::from("{}").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedVariable,
                start: 1,
                end: 2,
            })
        );
    }

    #[test]
    fn unmatched_expr_end_error() {
        assert_eq!(
            Parser::from("}").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::UnmatchedExprEnd,
                start: 0,
                end: 1,
            })
        );
    }

    #[test]
    fn unterminated_expr_start_after_variable_error() {
        assert_eq!(
            Parser::from("{f").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::UnmatchedExprStart,
                start: 0,
                end: 1,
            })
        );
    }

    #[test]
    fn variable() {
        assert_eq!(
            Parser::from("{f}").parse_items(),
            Ok(vec![Parsed {
                value: PatternItem::Expression {
                    variable: Parsed {
                        value: Variable::Filename,
                        start: 1,
                        end: 2,
                    },
                    filters: Vec::new(),
                },
                start: 0,
                end: 3,
            }])
        );
    }

    #[test]
    fn unknown_variable_error() {
        assert_eq!(
            Parser::from("{x}").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::UnknownVariable(Char::Raw('x')),
                start: 1,
                end: 2,
            })
        );
    }

    #[test]
    fn expr_start_inside_expr_error() {
        assert_eq!(
            Parser::from("{f{").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExprStartInsideExpr,
                start: 2,
                end: 3,
            })
        );
    }

    #[test]
    fn expected_pipe_or_expr_end_after_variable_error() {
        assert_eq!(
            Parser::from("{fg").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedPipeOrExprEnd,
                start: 2,
                end: 3,
            })
        );
    }

    #[test]
    fn expected_filter_but_end_error() {
        assert_eq!(
            Parser::from("{f|").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedFilter,
                start: 3,
                end: 3,
            })
        );
    }

    #[test]
    fn expected_filter_but_pipe_error() {
        assert_eq!(
            Parser::from("{f||").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedFilter,
                start: 3,
                end: 4,
            })
        );
    }

    #[test]
    fn expected_filter_but_expr_end_error() {
        assert_eq!(
            Parser::from("{f|}").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedFilter,
                start: 3,
                end: 4,
            })
        );
    }

    #[test]
    fn unternimeted_expr_start_after_filter_error() {
        assert_eq!(
            Parser::from("{f|l").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::UnmatchedExprStart,
                start: 0,
                end: 1,
            })
        );
    }

    #[test]
    fn expected_pipe_or_expr_end_after_filter_error() {
        assert_eq!(
            Parser::from("{f|ll").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedPipeOrExprEnd,
                start: 4,
                end: 5,
            })
        );
    }

    #[test]
    fn variable_single_filter() {
        assert_eq!(
            Parser::from("{b|l}").parse_items(),
            Ok(vec![Parsed {
                value: PatternItem::Expression {
                    variable: Parsed {
                        value: Variable::Basename,
                        start: 1,
                        end: 2,
                    },
                    filters: vec![Parsed {
                        value: Filter::Lowercase,
                        start: 3,
                        end: 4,
                    }],
                },
                start: 0,
                end: 5,
            }])
        );
    }

    #[test]
    fn variable_multiple_filters() {
        assert_eq!(
            Parser::from("{e|t|n1-3}").parse_items(),
            Ok(vec![Parsed {
                value: PatternItem::Expression {
                    variable: Parsed {
                        value: Variable::Extension,
                        start: 1,
                        end: 2,
                    },
                    filters: vec![
                        Parsed {
                            value: Filter::Trim,
                            start: 3,
                            end: 4,
                        },
                        Parsed {
                            value: Filter::Substring(Range::FromTo(0, 3)),
                            start: 5,
                            end: 9,
                        },
                    ],
                },
                start: 0,
                end: 10,
            }])
        );
    }

    #[test]
    fn invalid_filter_error() {
        assert_eq!(
            Parser::from("{f|n2-1}").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::RangeStartOverEnd(2, 1),
                start: 4,
                end: 7,
            })
        );
    }

    #[test]
    fn complex_input() {
        assert_eq!(
            Parser::from("image_{c|<000}.{e|l|r_e}2").parse_items(),
            Ok(vec![
                Parsed {
                    value: PatternItem::Constant("image_".to_string()),
                    start: 0,
                    end: 6,
                },
                Parsed {
                    value: PatternItem::Expression {
                        variable: Parsed {
                            value: Variable::LocalCounter,
                            start: 7,
                            end: 8,
                        },
                        filters: vec![Parsed {
                            value: Filter::LeftPad("000".to_string()),
                            start: 9,
                            end: 13,
                        }],
                    },
                    start: 6,
                    end: 14,
                },
                Parsed {
                    value: PatternItem::Constant(".".to_string()),
                    start: 14,
                    end: 15,
                },
                Parsed {
                    value: PatternItem::Expression {
                        variable: Parsed {
                            value: Variable::Extension,
                            start: 16,
                            end: 17,
                        },
                        filters: vec![
                            Parsed {
                                value: Filter::Lowercase,
                                start: 18,
                                end: 19,
                            },
                            Parsed {
                                value: Filter::ReplaceFirst(Substitution {
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
                },
                Parsed {
                    value: PatternItem::Constant("2".to_string()),
                    start: 24,
                    end: 25,
                },
            ])
        );
    }
}
