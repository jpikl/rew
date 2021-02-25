use crate::pattern::char::{AsChar, Char, Chars};
use crate::pattern::filter::Filter;
use crate::pattern::lexer::{Lexer, Token};
use crate::pattern::parse::{Error, ErrorKind, Parsed, Result};
use crate::pattern::reader::Reader;
use std::fmt;
use std::ops::Range;

#[derive(Debug, PartialEq)]
pub enum Item {
    Constant(String),
    Expression(Vec<Parsed<Filter>>),
}

impl fmt::Display for Item {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Constant(value) => write!(formatter, "Constant '{}'", value),
            Self::Expression(filters) if filters.is_empty() => {
                write!(formatter, "Empty expression")
            }
            Self::Expression(filters) if filters.len() == 1 => {
                write!(formatter, "Expression with a filter")
            }
            Self::Expression(filters) => {
                write!(formatter, "Expression with {} filters", filters.len())
            }
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
                    value: Item::Constant(Chars::from(&raw[..]).to_string()),
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
        let filters = self.parse_filters()?;
        let end = self.token_range().end;

        Ok(Some(Parsed {
            value: Item::Expression(filters),
            range: start..end,
        }))
    }

    fn parse_filters(&mut self) -> Result<Vec<Parsed<Filter>>> {
        let mut filters: Vec<Parsed<Filter>> = Vec::new();

        while let Some(token) = self.fetch_token()? {
            match &token.value {
                Token::Raw(raw) => {
                    filters.push(Self::parse_filter(&raw, &token.range)?);
                }
                Token::Pipe => {
                    if filters.is_empty() {
                        return Err(Error {
                            kind: ErrorKind::ExpectedFilterOrExprEnd,
                            range: token.range.clone(),
                        });
                    } else {
                        let position = self.token_range().end;
                        if let Some(token) = self.fetch_token()? {
                            if let Token::Raw(raw) = &token.value {
                                filters.push(Self::parse_filter(&raw, &token.range)?)
                            } else {
                                return Err(Error {
                                    kind: ErrorKind::ExpectedFilter,
                                    range: token.range.clone(),
                                });
                            }
                        } else {
                            return Err(Error {
                                kind: ErrorKind::ExpectedFilter,
                                range: position..position,
                            });
                        }
                    }
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
            }
        }

        Ok(filters)
    }

    fn parse_filter(chars: &[Char], range: &Range<usize>) -> Result<Parsed<Filter>> {
        let mut reader = Reader::new(Vec::from(chars));

        let filter = Filter::parse(&mut reader).map_err(|mut error| {
            let start = range.start + error.range.start;
            let end = range.start + error.range.end;

            error.range = start..end;
            error
        })?;

        if let Some(char) = reader.peek() {
            // There should be no remaining characters
            let start = range.start + reader.position();
            let end = range.start + reader.position() + char.len_utf8();

            Err(Error {
                kind: ErrorKind::ExpectedPipeOrExprEnd,
                range: start..end,
            })
        } else {
            Ok(Parsed {
                value: filter,
                range: range.clone(),
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
    use crate::pattern::index::IndexRange;
    use crate::pattern::padding::Padding;
    use crate::pattern::repetition::Repetition;
    use crate::pattern::substitution::Substitution;
    use crate::pattern::testing::make_parsed;

    mod item_display {
        use super::*;

        #[test]
        fn constant() {
            assert_eq!(
                Item::Constant(String::from("abc")).to_string(),
                "Constant 'abc'"
            );
        }

        #[test]
        fn empty_expression() {
            assert_eq!(Item::Expression(Vec::new()).to_string(), "Empty expression");
        }

        #[test]
        fn single_filter_expression() {
            assert_eq!(
                Item::Expression(vec![make_parsed(Filter::ToUppercase)]).to_string(),
                "Expression with a filter"
            );
        }

        #[test]
        fn multi_filter_expression() {
            assert_eq!(
                Item::Expression(vec![
                    make_parsed(Filter::ToUppercase),
                    make_parsed(Filter::Trim)
                ])
                .to_string(),
                "Expression with 2 filters"
            );
        }
    }

    mod parse {
        use super::*;

        #[test]
        fn empty() {
            assert_eq!(Parser::from("").parse_items(), Ok(Vec::new()));
        }

        #[test]
        fn constant() {
            assert_eq!(
                Parser::from("a").parse_items(),
                Ok(vec![Parsed {
                    value: Item::Constant(String::from("a")),
                    range: 0..1,
                }])
            );
        }

        #[test]
        fn pipe_outside_expr() {
            assert_eq!(
                Parser::from("|").parse_items(),
                Err(Error {
                    kind: ErrorKind::PipeOutsideExpr,
                    range: 0..1,
                })
            );
        }

        #[test]
        fn unmatched_expr_end() {
            assert_eq!(
                Parser::from("}").parse_items(),
                Err(Error {
                    kind: ErrorKind::UnmatchedExprEnd,
                    range: 0..1,
                })
            );
        }

        #[test]
        fn unmatched_expr_start() {
            assert_eq!(
                Parser::from("{").parse_items(),
                Err(Error {
                    kind: ErrorKind::UnmatchedExprStart,
                    range: 0..1,
                })
            );
        }

        #[test]
        fn filter_after_expr_start() {
            assert_eq!(
                Parser::from("{|").parse_items(),
                Err(Error {
                    kind: ErrorKind::ExpectedFilterOrExprEnd,
                    range: 1..2,
                })
            );
        }

        #[test]
        fn empty_expr() {
            assert_eq!(
                Parser::from("{}").parse_items(),
                Ok(vec![Parsed {
                    value: Item::Expression(Vec::new()),
                    range: 0..2,
                }])
            );
        }

        #[test]
        fn missing_pipe_or_expr_end() {
            assert_eq!(
                Parser::from("{f").parse_items(),
                Err(Error {
                    kind: ErrorKind::UnmatchedExprStart,
                    range: 0..1,
                })
            );
        }

        #[test]
        fn expr_start_after_filter() {
            assert_eq!(
                Parser::from("{f{").parse_items(),
                Err(Error {
                    kind: ErrorKind::ExprStartInsideExpr,
                    range: 2..3,
                })
            );
        }

        #[test]
        fn expr_single_filter() {
            assert_eq!(
                Parser::from("{f}").parse_items(),
                Ok(vec![Parsed {
                    value: Item::Expression(vec![Parsed {
                        value: Filter::FileName,
                        range: 1..2,
                    }]),
                    range: 0..3,
                }])
            );
        }

        #[test]
        fn filter_after_filter() {
            assert_eq!(
                Parser::from("{fg").parse_items(),
                Err(Error {
                    kind: ErrorKind::ExpectedPipeOrExprEnd,
                    range: 2..3,
                })
            );
        }

        #[test]
        fn missing_filter_after_pipe() {
            assert_eq!(
                Parser::from("{f|").parse_items(),
                Err(Error {
                    kind: ErrorKind::ExpectedFilter,
                    range: 3..3,
                })
            );
        }

        #[test]
        fn pipe_after_pipe() {
            assert_eq!(
                Parser::from("{f||").parse_items(),
                Err(Error {
                    kind: ErrorKind::ExpectedFilter,
                    range: 3..4,
                })
            );
        }

        #[test]
        fn expr_end_after_pipe() {
            assert_eq!(
                Parser::from("{f|}").parse_items(),
                Err(Error {
                    kind: ErrorKind::ExpectedFilter,
                    range: 3..4,
                })
            );
        }

        #[test]
        fn missing_pipe_or_expr_end_2() {
            assert_eq!(
                Parser::from("{f|v").parse_items(),
                Err(Error {
                    kind: ErrorKind::UnmatchedExprStart,
                    range: 0..1,
                })
            );
        }

        #[test]
        fn filter_after_filter_2() {
            assert_eq!(
                Parser::from("{f|vv").parse_items(),
                Err(Error {
                    kind: ErrorKind::ExpectedPipeOrExprEnd,
                    range: 4..5,
                })
            );
        }

        #[test]
        fn invalid_filter() {
            assert_eq!(
                Parser::from("{#2-1}").parse_items(),
                Err(Error {
                    kind: ErrorKind::RangeStartOverEnd(String::from("2"), String::from("1")),
                    range: 2..5,
                })
            );
        }

        #[test]
        fn expr_multiple_filters() {
            assert_eq!(
                Parser::from("{e|t|#1-3}").parse_items(),
                Ok(vec![Parsed {
                    value: Item::Expression(vec![
                        Parsed {
                            value: Filter::Extension,
                            range: 1..2,
                        },
                        Parsed {
                            value: Filter::Trim,
                            range: 3..4,
                        },
                        Parsed {
                            value: Filter::Substring(IndexRange::new(0, Some(2))),
                            range: 5..9,
                        },
                    ]),
                    range: 0..10,
                }])
            );
        }

        #[test]
        fn complex_pattern() {
            assert_eq!(
                Parser::from("image_{c|<3:0}.{e|v|r_e}2").parse_items(),
                Ok(vec![
                    Parsed {
                        value: Item::Constant(String::from("image_")),
                        range: 0..6,
                    },
                    Parsed {
                        value: Item::Expression(vec![
                            Parsed {
                                value: Filter::LocalCounter,
                                range: 7..8,
                            },
                            Parsed {
                                value: Filter::LeftPad(Padding::Repeated(Repetition {
                                    count: 3,
                                    value: String::from("0")
                                })),
                                range: 9..13,
                            }
                        ]),
                        range: 6..14,
                    },
                    Parsed {
                        value: Item::Constant(String::from(".")),
                        range: 14..15,
                    },
                    Parsed {
                        value: Item::Expression(vec![
                            Parsed {
                                value: Filter::Extension,
                                range: 16..17,
                            },
                            Parsed {
                                value: Filter::ToLowercase,
                                range: 18..19,
                            },
                            Parsed {
                                value: Filter::ReplaceFirst(Substitution {
                                    target: 'e'.to_string(),
                                    replacement: String::new(),
                                }),
                                range: 20..23,
                            },
                        ]),
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
}
