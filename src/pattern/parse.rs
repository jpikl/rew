use crate::pattern::error::{ParseError, ParseErrorKind};
use crate::pattern::lexer::Lexer;
use crate::pattern::parser::Parser;
use crate::pattern::Pattern;

#[derive(Debug, PartialEq)]
pub struct Parsed<T> {
    pub value: T,
    pub start: usize,
    pub end: usize,
}

pub type ParseResult<T> = Result<T, ParseError>;

impl Pattern {
    pub fn parse(string: &str) -> ParseResult<Self> {
        Self::parse_tokens(Lexer::from(string))
    }

    pub fn parse_tokens(lexer: Lexer) -> ParseResult<Self> {
        let mut parser = Parser::new(lexer);
        let mut items = Vec::new();

        while let Some(item) = parser.parse_item()? {
            items.push(item);
        }

        if items.is_empty() {
            Err(ParseError {
                kind: ParseErrorKind::ExpectedPattern,
                start: 0,
                end: 0,
            })
        } else {
            Ok(Self { items })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::parser::PatternItem;
    use crate::pattern::variable::Variable;

    #[test]
    fn empty_error() {
        assert_parse_error(
            "",
            ParseError {
                kind: ParseErrorKind::ExpectedPattern,
                start: 0,
                end: 0,
            },
        );
    }

    #[test]
    fn single_item() {
        assert_parse_items(
            "a",
            vec![Parsed {
                value: PatternItem::Constant("a".to_string()),
                start: 0,
                end: 1,
            }],
        );
    }

    #[test]
    fn multiple_items() {
        assert_parse_items(
            "a{E}",
            vec![
                Parsed {
                    value: PatternItem::Constant("a".to_string()),
                    start: 0,
                    end: 1,
                },
                Parsed {
                    value: PatternItem::Expression {
                        variable: Parsed {
                            value: Variable::ExtensionWithDot,
                            start: 2,
                            end: 3,
                        },
                        transforms: Vec::new(),
                    },
                    start: 1,
                    end: 4,
                },
            ],
        );
    }

    #[test]
    fn input_error() {
        assert_parse_error(
            "a{E",
            ParseError {
                kind: ParseErrorKind::UnterminatedExprStart,
                start: 1,
                end: 2,
            },
        );
    }

    fn assert_parse_items(string: &str, items: Vec<Parsed<PatternItem>>) {
        assert_eq!(Pattern::parse(string), Ok(Pattern { items }));
    }

    fn assert_parse_error(string: &str, error: ParseError) {
        assert_eq!(Pattern::parse(string), Err(error));
    }
}
