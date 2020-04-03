use crate::pattern::error::{EvalError, ParseError};
use crate::pattern::parser::Parser;
use crate::pattern::transform::Transform;
use crate::pattern::variable::Variable;
use std::path::{Path, PathBuf};

mod error;
mod lexer;
mod number;
mod parser;
mod range;
mod reader;
mod substitution;
mod transform;
mod variable;

#[derive(Debug, PartialEq)]
struct Pattern {
    items: Vec<Parsed<PatternItem>>,
}

#[derive(Debug, PartialEq)]
enum PatternItem {
    Constant(String),
    Expression {
        variable: Parsed<Variable>,
        transforms: Vec<Parsed<Transform>>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct EvalContext<'a> {
    path: &'a Path,
    local_counter: u32,
    global_counter: u32,
    capture_groups: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct Parsed<T> {
    value: T,
    start: usize,
    end: usize,
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
                position: 0,
            })
        } else {
            Ok(Self { items })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::ParseIntError;

    #[test]
    fn parse_empty_error() {
        assert_parse_error(
            "",
            ParseError {
                message: "Empty pattern",
                position: 0,
            },
        );
    }

    #[test]
    fn parse_single_item() {
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
    fn parse_multiple_items() {
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
    fn parse_error() {
        assert_parse_error(
            "a{E",
            ParseError {
                message: "Expected pipe or expression end",
                position: 3,
            },
        );
    }

    fn assert_parse_error(string: &str, error: ParseError) {
        assert_eq!(Pattern::parse(string), Err(error));
    }

    fn assert_parse_items(string: &str, items: Vec<Parsed<PatternItem>>) {
        assert_eq!(Pattern::parse(string), Ok(Pattern { items }));
    }
}
