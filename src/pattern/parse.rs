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
    pub message: &'static str,
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
                message: "Empty pattern",
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
    use crate::pattern::variable::Variable;
    use crate::pattern::PatternItem;

    #[test]
    fn empty_error() {
        assert_parse_error(
            "",
            ParseError {
                message: "Empty pattern",
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
                message: "Expected pipe or expression end",
                start: 3,
                end: 3,
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
