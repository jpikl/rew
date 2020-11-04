use crate::pattern::char::{AsChar, Char};
use crate::pattern::filter::Filter;
use crate::pattern::lexer::{Lexer, Token};
use crate::pattern::parse::{Error, ErrorKind, Parsed, Result};
use crate::pattern::reader::Reader;
use crate::pattern::variable::Variable;
use std::fmt;
use std::ops::Range;

#[derive(Debug, PartialEq)]
pub enum Item {
    Constant(String),
    Expression {
        variable: Parsed<Variable>,
        filters: Vec<Parsed<Filter>>,
    },
}

impl fmt::Display for Item {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Constant(value) => write!(formatter, "Constant '{}'", value),
            Self::Expression { filters, .. } if filters.is_empty() => {
                write!(formatter, "Expression with a variable")
            }
            Self::Expression { filters, .. } if filters.len() == 1 => {
                write!(formatter, "Expression with a variable and a filter")
            }
            Self::Expression { filters, .. } => write!(
                formatter,
                "Expression with a variable and {} filters",
                filters.len()
            ),
        }
    }
}

pub struct Parser {
    lexer: Lexer,
    token: Option<Parsed<Token>>,
}

impl From<&str> for Parser {
    fn from(string: &str) -> Self {
        Self::new(Lexer::new(string))
    }
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self { lexer, token: None }
    }

    pub fn parse_items(&mut self) -> Result<Vec<Parsed<Item>>> {
        let mut items = Vec::new();

        while let Some(item) = self.parse_item()? {
            items.push(item);
        }

        Ok(items)
    }

    fn parse_item(&mut self) -> Result<Option<Parsed<Item>>> {
        if let Some(token) = self.fetch_token()? {
            match &token.value {
                Token::Raw(raw) => Ok(Some(Parsed {
                    value: Item::Constant(Char::join(raw)),
                    range: token.range.clone(),
                })),
                Token::ExprStart => {
                    let expr_start_range = token.range.clone();
                    let expression = self.parse_expression()?;

                    if let Some(Token::ExprEnd) = self.token_value() {
                        Ok(expression)
                    } else {
                        Err(Error {
                            kind: ErrorKind::UnmatchedExprStart,
                            range: expr_start_range,
                        })
                    }
                }
                Token::ExprEnd => Err(Error {
                    kind: ErrorKind::UnmatchedExprEnd,
                    range: token.range.clone(),
                }),
                Token::Pipe => Err(Error {
                    kind: ErrorKind::PipeOutsideExpr,
                    range: token.range.clone(),
                }),
            }
        } else {
            Ok(None)
        }
    }

    fn parse_expression(&mut self) -> Result<Option<Parsed<Item>>> {
        let start = self.token_range().start;
        let variable = self.parse_variable()?;
        let filters = self.parse_filters()?;
        let end = self.token_range().end;

        Ok(Some(Parsed {
            value: Item::Expression { variable, filters },
            range: start..end,
        }))
    }

    fn parse_variable(&mut self) -> Result<Parsed<Variable>> {
        self.parse_expression_member(Variable::parse, ErrorKind::ExpectedVariable)
    }

    fn parse_filters(&mut self) -> Result<Vec<Parsed<Filter>>> {
        let mut filters: Vec<Parsed<Filter>> = Vec::new();

        while let Some(token) = self.fetch_token()? {
            match token.value {
                Token::Pipe => {
                    filters.push(self.parse_filter()?);
                }
                Token::ExprStart => {
                    return Err(Error {
                        kind: ErrorKind::ExprStartInsideExpr,
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

    fn parse_filter(&mut self) -> Result<Parsed<Filter>> {
        self.parse_expression_member(Filter::parse, ErrorKind::ExpectedFilter)
    }

    fn parse_expression_member<T, F: FnOnce(&mut Reader<Char>) -> Result<T>>(
        &mut self,
        parse: F,
        error_kind: ErrorKind,
    ) -> Result<Parsed<T>> {
        let position = self.token_range().end;
        let token = self.fetch_token()?.ok_or_else(|| Error {
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
                let end = position + reader.position() + char.len_utf8();

                Err(Error {
                    kind: ErrorKind::ExpectedPipeOrExprEnd,
                    range: start..end,
                })
            } else {
                Ok(Parsed {
                    value,
                    range: token.range.clone(),
                })
            }
        } else {
            Err(Error {
                kind: error_kind,
                range: token.range.clone(),
            })
        }
    }

    fn fetch_token(&mut self) -> Result<Option<&Parsed<Token>>> {
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
    use crate::pattern::testing::make_parsed;

    #[test]
    fn item_fmt() {
        assert_eq!(
            Item::Constant(String::from("abc")).to_string(),
            "Constant 'abc'"
        );

        assert_eq!(
            (Item::Expression {
                variable: make_parsed(Variable::InputPath),
                filters: Vec::new()
            })
            .to_string(),
            "Expression with a variable"
        );

        assert_eq!(
            (Item::Expression {
                variable: make_parsed(Variable::InputPath),
                filters: vec![make_parsed(Filter::ToUppercase)]
            })
            .to_string(),
            "Expression with a variable and a filter"
        );

        assert_eq!(
            (Item::Expression {
                variable: make_parsed(Variable::InputPath),
                filters: vec![make_parsed(Filter::ToUppercase), make_parsed(Filter::Trim)]
            })
            .to_string(),
            "Expression with a variable and 2 filters"
        );
    }

    #[test]
    fn parse_empty() {
        assert_eq!(Parser::from("").parse_items(), Ok(Vec::new()));
    }

    #[test]
    fn parse_constant() {
        assert_eq!(
            Parser::from("a").parse_items(),
            Ok(vec![Parsed {
                value: Item::Constant(String::from("a")),
                range: 0..1,
            }])
        );
    }

    #[test]
    fn parse_expected_variable_but_end_error() {
        assert_eq!(
            Parser::from("{").parse_items(),
            Err(Error {
                kind: ErrorKind::ExpectedVariable,
                range: 1..1,
            })
        );
    }

    #[test]
    fn parse_expected_variable_but_pipe_error() {
        assert_eq!(
            Parser::from("{|").parse_items(),
            Err(Error {
                kind: ErrorKind::ExpectedVariable,
                range: 1..2,
            })
        );
    }

    #[test]
    fn parse_pipe_outside_expr_error() {
        assert_eq!(
            Parser::from("|").parse_items(),
            Err(Error {
                kind: ErrorKind::PipeOutsideExpr,
                range: 0..1,
            })
        );
    }

    #[test]
    fn parse_expected_variable_but_expr_end_error() {
        assert_eq!(
            Parser::from("{}").parse_items(),
            Err(Error {
                kind: ErrorKind::ExpectedVariable,
                range: 1..2,
            })
        );
    }

    #[test]
    fn parse_unmatched_expr_end_error() {
        assert_eq!(
            Parser::from("}").parse_items(),
            Err(Error {
                kind: ErrorKind::UnmatchedExprEnd,
                range: 0..1,
            })
        );
    }

    #[test]
    fn parse_unterminated_expr_start_after_variable_error() {
        assert_eq!(
            Parser::from("{f").parse_items(),
            Err(Error {
                kind: ErrorKind::UnmatchedExprStart,
                range: 0..1,
            })
        );
    }

    #[test]
    fn parse_variable() {
        assert_eq!(
            Parser::from("{f}").parse_items(),
            Ok(vec![Parsed {
                value: Item::Expression {
                    variable: Parsed {
                        value: Variable::FileName,
                        range: 1..2,
                    },
                    filters: Vec::new(),
                },
                range: 0..3,
            }])
        );
    }

    #[test]
    fn parse_unknown_variable_error() {
        assert_eq!(
            Parser::from("{x}").parse_items(),
            Err(Error {
                kind: ErrorKind::UnknownVariable(Char::Raw('x')),
                range: 1..2,
            })
        );
    }

    #[test]
    fn parse_expr_start_inside_expr_error() {
        assert_eq!(
            Parser::from("{f{").parse_items(),
            Err(Error {
                kind: ErrorKind::ExprStartInsideExpr,
                range: 2..3,
            })
        );
    }

    #[test]
    fn parse_expected_pipe_or_expr_end_after_variable_error() {
        assert_eq!(
            Parser::from("{fg").parse_items(),
            Err(Error {
                kind: ErrorKind::ExpectedPipeOrExprEnd,
                range: 2..3,
            })
        );
    }

    #[test]
    fn parse_expected_filter_but_end_error() {
        assert_eq!(
            Parser::from("{f|").parse_items(),
            Err(Error {
                kind: ErrorKind::ExpectedFilter,
                range: 3..3,
            })
        );
    }

    #[test]
    fn parse_expected_filter_but_pipe_error() {
        assert_eq!(
            Parser::from("{f||").parse_items(),
            Err(Error {
                kind: ErrorKind::ExpectedFilter,
                range: 3..4,
            })
        );
    }

    #[test]
    fn parse_expected_filter_but_expr_end_error() {
        assert_eq!(
            Parser::from("{f|}").parse_items(),
            Err(Error {
                kind: ErrorKind::ExpectedFilter,
                range: 3..4,
            })
        );
    }

    #[test]
    fn parse_unternimeted_expr_start_after_filter_error() {
        assert_eq!(
            Parser::from("{f|l").parse_items(),
            Err(Error {
                kind: ErrorKind::UnmatchedExprStart,
                range: 0..1,
            })
        );
    }

    #[test]
    fn parse_expected_pipe_or_expr_end_after_filter_error() {
        assert_eq!(
            Parser::from("{f|ll").parse_items(),
            Err(Error {
                kind: ErrorKind::ExpectedPipeOrExprEnd,
                range: 4..5,
            })
        );
    }

    #[test]
    fn parse_variable_single_filter() {
        assert_eq!(
            Parser::from("{b|l}").parse_items(),
            Ok(vec![Parsed {
                value: Item::Expression {
                    variable: Parsed {
                        value: Variable::BaseName,
                        range: 1..2,
                    },
                    filters: vec![Parsed {
                        value: Filter::ToLowercase,
                        range: 3..4,
                    }],
                },
                range: 0..5,
            }])
        );
    }

    #[test]
    fn parse_variable_multiple_filters() {
        assert_eq!(
            Parser::from("{e|t|n1-3}").parse_items(),
            Ok(vec![Parsed {
                value: Item::Expression {
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
    fn parse_invalid_filter_error() {
        assert_eq!(
            Parser::from("{f|n2-1}").parse_items(),
            Err(Error {
                kind: ErrorKind::RangeStartOverEnd(2, 1),
                range: 4..7,
            })
        );
    }

    #[test]
    fn parse_complex_input() {
        assert_eq!(
            Parser::from("image_{c|<000}.{e|l|r_e}2").parse_items(),
            Ok(vec![
                Parsed {
                    value: Item::Constant(String::from("image_")),
                    range: 0..6,
                },
                Parsed {
                    value: Item::Expression {
                        variable: Parsed {
                            value: Variable::LocalCounter,
                            range: 7..8,
                        },
                        filters: vec![Parsed {
                            value: Filter::LeftPad(String::from("000")),
                            range: 9..13,
                        }],
                    },
                    range: 6..14,
                },
                Parsed {
                    value: Item::Constant(String::from(".")),
                    range: 14..15,
                },
                Parsed {
                    value: Item::Expression {
                        variable: Parsed {
                            value: Variable::Extension,
                            range: 16..17,
                        },
                        filters: vec![
                            Parsed {
                                value: Filter::ToLowercase,
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
                    value: Item::Constant(String::from("2")),
                    range: 24..25,
                },
            ])
        );
    }
}
