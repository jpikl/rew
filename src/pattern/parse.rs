use crate::pattern::error::ErrorType;
use crate::pattern::parser::Parser;
use crate::pattern::Pattern;

#[derive(Debug, PartialEq)]
pub struct Parsed<T> {
    pub value: T,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub typ: ErrorType,
    pub start: usize,
    pub end: usize,
}

impl Pattern {
    pub fn parse(string: &str) -> Result<Self, ParseError> {
        let mut parser = Parser::new(string);
        let mut items = Vec::new();

        while let Some(item) = parser.parse_item()? {
            items.push(item);
        }

        if items.is_empty() {
            Err(ParseError {
                typ: ErrorType::ExpectedPattern,
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
                typ: ErrorType::ExpectedPattern,
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
                typ: ErrorType::UnterminatedExprStart,
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
