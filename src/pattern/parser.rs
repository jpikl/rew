use crate::pattern::char::Char;
use crate::pattern::filter::Filter;
use crate::pattern::lexer::{Lexer, Token};
use crate::pattern::parse::{ParseError, ParseErrorKind, ParseResult, Parsed};
use crate::pattern::reader::Reader;
use crate::pattern::variable::Variable;
use std::ops::Range;

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
                    range: token.range.clone(),
                })),
                Token::ExprStart => {
                    let expr_start_range = token.range.clone();
                    let expression = self.parse_expression()?;

                    if let Some(Token::ExprEnd) = self.token_value() {
                        Ok(expression)
                    } else {
                        Err(ParseError {
                            kind: ParseErrorKind::UnmatchedExprStart,
                            range: expr_start_range,
                        })
                    }
                }
                Token::ExprEnd => Err(ParseError {
                    kind: ParseErrorKind::UnmatchedExprEnd,
                    range: token.range.clone(),
                }),
                Token::Pipe => Err(ParseError {
                    kind: ParseErrorKind::PipeOutsideExpr,
                    range: token.range.clone(),
                }),
            }
        } else {
            Ok(None)
        }
    }

    fn parse_expression(&mut self) -> ParseResult<Option<Parsed<PatternItem>>> {
        let start = self.token_range().start;
        let variable = self.parse_variable()?;
        let filters = self.parse_filters()?;
        let end = self.token_range().end;

        Ok(Some(Parsed {
            value: PatternItem::Expression { variable, filters },
            range: start..end,
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
                        range: token.range.clone(),
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
        let position = self.token_range().end;
        let token = self.fetch_token()?.ok_or_else(|| ParseError {
            kind: error_kind.clone(),
            range: position..position,
        })?;
        if let Token::Raw(raw) = &token.value {
            let mut reader = Reader::new(raw.clone());

            let value = parse(&mut reader).map_err(|mut error| {
                let start = error.range.start + position;
                let end = error.range.end + position;

                error.range = start..end;
                error
            })?;

            if let Some(char) = reader.peek() {
                // There should be no remaining characters
                let start = position + reader.position();
                let end = position + reader.position() + char.len();

                Err(ParseError {
                    kind: ParseErrorKind::ExpectedPipeOrExprEnd,
                    range: start..end,
                })
            } else {
                Ok(Parsed {
                    value,
                    range: token.range.clone(),
                })
            }
        } else {
            Err(ParseError {
                kind: error_kind,
                range: token.range.clone(),
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

    fn token_range(&self) -> &Range<usize> {
        self.token.as_ref().map_or(&(0..0), |token| &token.range)
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
                range: 0..1,
            }])
        );
    }

    #[test]
    fn expected_variable_but_end_error() {
        assert_eq!(
            Parser::from("{").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedVariable,
                range: 1..1,
            })
        );
    }

    #[test]
    fn expected_variable_but_pipe_error() {
        assert_eq!(
            Parser::from("{|").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedVariable,
                range: 1..2,
            })
        );
    }

    #[test]
    fn pipe_outside_expr_error() {
        assert_eq!(
            Parser::from("|").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::PipeOutsideExpr,
                range: 0..1,
            })
        );
    }

    #[test]
    fn expected_variable_but_expr_end_error() {
        assert_eq!(
            Parser::from("{}").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedVariable,
                range: 1..2,
            })
        );
    }

    #[test]
    fn unmatched_expr_end_error() {
        assert_eq!(
            Parser::from("}").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::UnmatchedExprEnd,
                range: 0..1,
            })
        );
    }

    #[test]
    fn unterminated_expr_start_after_variable_error() {
        assert_eq!(
            Parser::from("{f").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::UnmatchedExprStart,
                range: 0..1,
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
                        range: 1..2,
                    },
                    filters: Vec::new(),
                },
                range: 0..3,
            }])
        );
    }

    #[test]
    fn unknown_variable_error() {
        assert_eq!(
            Parser::from("{x}").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::UnknownVariable(Char::Raw('x')),
                range: 1..2,
            })
        );
    }

    #[test]
    fn expr_start_inside_expr_error() {
        assert_eq!(
            Parser::from("{f{").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExprStartInsideExpr,
                range: 2..3,
            })
        );
    }

    #[test]
    fn expected_pipe_or_expr_end_after_variable_error() {
        assert_eq!(
            Parser::from("{fg").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedPipeOrExprEnd,
                range: 2..3,
            })
        );
    }

    #[test]
    fn expected_filter_but_end_error() {
        assert_eq!(
            Parser::from("{f|").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedFilter,
                range: 3..3,
            })
        );
    }

    #[test]
    fn expected_filter_but_pipe_error() {
        assert_eq!(
            Parser::from("{f||").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedFilter,
                range: 3..4,
            })
        );
    }

    #[test]
    fn expected_filter_but_expr_end_error() {
        assert_eq!(
            Parser::from("{f|}").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedFilter,
                range: 3..4,
            })
        );
    }

    #[test]
    fn unternimeted_expr_start_after_filter_error() {
        assert_eq!(
            Parser::from("{f|l").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::UnmatchedExprStart,
                range: 0..1,
            })
        );
    }

    #[test]
    fn expected_pipe_or_expr_end_after_filter_error() {
        assert_eq!(
            Parser::from("{f|ll").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedPipeOrExprEnd,
                range: 4..5,
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
                        range: 1..2,
                    },
                    filters: vec![Parsed {
                        value: Filter::Lowercase,
                        range: 3..4,
                    }],
                },
                range: 0..5,
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
                        range: 1..2,
                    },
                    filters: vec![
                        Parsed {
                            value: Filter::Trim,
                            range: 3..4,
                        },
                        Parsed {
                            value: Filter::Substring(Range::FromTo(0, 3)),
                            range: 5..9,
                        },
                    ],
                },
                range: 0..10,
            }])
        );
    }

    #[test]
    fn invalid_filter_error() {
        assert_eq!(
            Parser::from("{f|n2-1}").parse_items(),
            Err(ParseError {
                kind: ParseErrorKind::RangeStartOverEnd(2, 1),
                range: 4..7,
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
                    range: 0..6,
                },
                Parsed {
                    value: PatternItem::Expression {
                        variable: Parsed {
                            value: Variable::LocalCounter,
                            range: 7..8,
                        },
                        filters: vec![Parsed {
                            value: Filter::LeftPad("000".to_string()),
                            range: 9..13,
                        }],
                    },
                    range: 6..14,
                },
                Parsed {
                    value: PatternItem::Constant(".".to_string()),
                    range: 14..15,
                },
                Parsed {
                    value: PatternItem::Expression {
                        variable: Parsed {
                            value: Variable::Extension,
                            range: 16..17,
                        },
                        filters: vec![
                            Parsed {
                                value: Filter::Lowercase,
                                range: 18..19,
                            },
                            Parsed {
                                value: Filter::ReplaceFirst(Substitution {
                                    value: 'e'.to_string(),
                                    replacement: String::new(),
                                }),
                                range: 20..23,
                            },
                        ],
                    },
                    range: 15..24,
                },
                Parsed {
                    value: PatternItem::Constant("2".to_string()),
                    range: 24..25,
                },
            ])
        );
    }
}
