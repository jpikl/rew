use crate::pattern::char::{AsChar, Char, Chars};
use crate::pattern::escape::escape_str;
use crate::pattern::filter::Filter;
use crate::pattern::lexer::{Lexer, ParsedToken, Token};
use crate::pattern::parse::{Config, Error, ErrorKind, Parsed, Result};
use crate::pattern::reader::Reader;
use crate::utils::ByteRange;
use std::fmt;

pub type ParsedFilter = Parsed<Filter>;
pub type ParsedItem = Parsed<Item>;

#[derive(Debug, PartialEq)]
pub enum Item {
    Constant(String),
    Expression(Vec<ParsedFilter>),
}

impl fmt::Display for Item {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Constant(value) => write!(formatter, "Constant '{}'", escape_str(&value)),
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

pub struct Parser<'a> {
    lexer: Lexer,
    token: Option<ParsedToken>,
    config: &'a Config,
}

impl<'a> Parser<'a> {
    pub fn new(input: &str, config: &'a Config) -> Self {
        Self {
            lexer: Lexer::new(input, config.escape),
            token: None,
            config,
        }
    }

    pub fn parse_items(&mut self) -> Result<Vec<ParsedItem>> {
        let mut items = Vec::new();

        while let Some(item) = self.parse_item()? {
            items.push(item);
        }

        Ok(items)
    }

    fn parse_item(&mut self) -> Result<Option<ParsedItem>> {
        self.fetch_token()?;

        if let Some(token) = &self.token {
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

    fn parse_expression(&mut self) -> Result<Option<ParsedItem>> {
        let start = self.token_range().start;
        let filters = self.parse_filters()?;
        let end = self.token_range().end;

        Ok(Some(Parsed {
            value: Item::Expression(filters),
            range: start..end,
        }))
    }

    fn parse_filters(&mut self) -> Result<Vec<ParsedFilter>> {
        let mut filters: Vec<ParsedFilter> = Vec::new();
        self.fetch_token()?;

        while let Some(token) = &self.token {
            match &token.value {
                Token::Raw(raw) => {
                    filters.push(self.parse_filter(&raw, &token.range)?);
                }
                Token::Pipe => {
                    if filters.is_empty() {
                        return Err(Error {
                            kind: ErrorKind::ExpectedFilterOrExprEnd,
                            range: token.range.clone(),
                        });
                    } else {
                        let position = self.token_range().end;
                        self.fetch_token()?;

                        if let Some(token) = &self.token {
                            if let Token::Raw(raw) = &token.value {
                                filters.push(self.parse_filter(&raw, &token.range)?)
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
            self.fetch_token()?;
        }

        Ok(filters)
    }

    fn parse_filter(&self, chars: &[Char], range: &ByteRange) -> Result<ParsedFilter> {
        let mut reader = Reader::new(Vec::from(chars));

        let filter = Filter::parse(&mut reader, self.config).map_err(|mut error| {
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

    fn fetch_token(&mut self) -> Result<()> {
        self.token = self.lexer.read_token()?;
        Ok(())
    }

    fn token_value(&self) -> Option<&Token> {
        self.token.as_ref().map(|token| &token.value)
    }

    fn token_range(&self) -> &ByteRange {
        self.token.as_ref().map_or(&(0..0), |token| &token.range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(Item::Constant("abc".into()),     "Constant 'abc'"            ; "constant")]
    #[test_case(Item::Expression(Vec::new()),     "Empty expression"          ; "empty expr")]
    #[test_case(Item::Expression(vec![f()]),      "Expression with a filter"  ; "expr single filter")]
    #[test_case(Item::Expression(vec![f(), f()]), "Expression with 2 filters" ; "expr multiple filters")]
    fn item_display(item: Item, result: &str) {
        assert_eq!(item.to_string(), result);
    }

    fn f() -> ParsedFilter {
        Parsed::from(Filter::Trim)
    }

    mod parse {
        use super::*;
        use crate::pattern::index::Index;
        use crate::pattern::padding::Padding;
        use crate::pattern::range::Range;
        use crate::pattern::repetition::Repetition;
        use crate::pattern::substitution::Substitution;
        use test_case::test_case;

        #[test_case("|",      0..1, ErrorKind::PipeOutsideExpr                           ; "pipe outside expr")]
        #[test_case("}",      0..1, ErrorKind::UnmatchedExprEnd                          ; "unmatched expr end")]
        #[test_case("{",      0..1, ErrorKind::UnmatchedExprStart                        ; "unmatched expr start")]
        #[test_case("{|",     1..2, ErrorKind::ExpectedFilterOrExprEnd                   ; "filter after expr start")]
        #[test_case("{f",     0..1, ErrorKind::UnmatchedExprStart                        ; "missing pipe or expr end")]
        #[test_case("{f{",    2..3, ErrorKind::ExprStartInsideExpr                       ; "expr start after filter")]
        #[test_case("{ff",    2..3, ErrorKind::ExpectedPipeOrExprEnd                     ; "filter after filter")]
        #[test_case("{f|",    3..3, ErrorKind::ExpectedFilter                            ; "missing filter after pipe")]
        #[test_case("{f||",   3..4, ErrorKind::ExpectedFilter                            ; "pipe after pipe")]
        #[test_case("{f|}",   3..4, ErrorKind::ExpectedFilter                            ; "expr end after pipe")]
        #[test_case("{f|f",   0..1, ErrorKind::UnmatchedExprStart                        ; "missing pipe or expr end 2")]
        #[test_case("{f|ff",  4..5, ErrorKind::ExpectedPipeOrExprEnd                     ; "filter after filter 2")]
        #[test_case("{#2-1}", 2..5, ErrorKind::RangeStartOverEnd("2".into(), "1".into()) ; "invalid filter")]
        fn err(input: &str, range: ByteRange, kind: ErrorKind) {
            assert_eq!(
                Parser::new(input, &Config::fixture()).parse_items(),
                Err(Error { kind, range })
            );
        }

        #[test_case("",                          Vec::new()              ; "empty ")]
        #[test_case("a",                         constant()              ; "constant ")]
        #[test_case("{}",                        empty_expr()            ; "empty expr ")]
        #[test_case("{f}",                       expr_single_filter()    ; "expr single filter ")]
        #[test_case("{e|t|#1-3}",                expr_multiple_filters() ; "expr multiple filters ")]
        #[test_case("image_{c|<3:0}.{e|v|r_e}2", complex_pattern()       ; "complex pattern ")]
        fn ok(input: &str, output: Vec<ParsedItem>) {
            assert_eq!(
                Parser::new(input, &Config::fixture()).parse_items(),
                Ok(output)
            );
        }

        fn constant() -> Vec<ParsedItem> {
            vec![Parsed {
                value: Item::Constant("a".into()),
                range: 0..1,
            }]
        }

        fn empty_expr() -> Vec<ParsedItem> {
            vec![Parsed {
                value: Item::Expression(Vec::new()),
                range: 0..2,
            }]
        }

        fn expr_single_filter() -> Vec<ParsedItem> {
            vec![Parsed {
                value: Item::Expression(vec![Parsed {
                    value: Filter::FileName,
                    range: 1..2,
                }]),
                range: 0..3,
            }]
        }

        fn expr_multiple_filters() -> Vec<ParsedItem> {
            vec![Parsed {
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
                        value: Filter::Substring(Range::<Index>(0, Some(3))),
                        range: 5..9,
                    },
                ]),
                range: 0..10,
            }]
        }

        fn complex_pattern() -> Vec<ParsedItem> {
            vec![
                Parsed {
                    value: Item::Constant("image_".into()),
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
                                value: Some("0".into()),
                            })),
                            range: 9..13,
                        },
                    ]),
                    range: 6..14,
                },
                Parsed {
                    value: Item::Constant(".".into()),
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
                    value: Item::Constant("2".into()),
                    range: 24..25,
                },
            ]
        }
    }
}
