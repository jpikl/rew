use crate::pattern::error::{EvalError, ParseError};
use crate::pattern::parser::Parser;
use crate::pattern::transform::Transform;
use crate::pattern::variable::Variable;
use std::path::Path;

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
pub enum PatternItem {
    Constant(String),
    Expression {
        variable: Parsed<Variable>,
        transforms: Vec<Parsed<Transform>>,
    },
}

#[derive(Debug, PartialEq)]
pub struct Parsed<T> {
    value: T,
    start: usize,
    end: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct EvalContext<'a> {
    path: &'a Path,
    local_counter: u32,
    global_counter: u32,
    capture_groups: Vec<String>,
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

    pub fn eval(&self, context: &mut EvalContext) -> Result<String, EvalError> {
        let mut output = String::new();

        for item in self.items.iter() {
            match &item.value {
                PatternItem::Constant(string) => output.push_str(string),
                PatternItem::Expression {
                    variable,
                    transforms,
                } => {
                    match variable.value.eval(context) {
                        Ok(mut string) => {
                            for transform in transforms.iter() {
                                string = transform.value.apply(string);
                            }
                            output.push_str(&string)
                        }
                        Err(message) => {
                            return Err(EvalError { message, variable });
                        }
                    };
                }
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn eval_constant() {
        let mut context = make_context();
        let pattern = Pattern::parse("abc").unwrap();
        pattern.assert_eval(&mut context, "abc");
    }

    #[test]
    fn eval_expression() {
        let mut context = make_context();
        let pattern = Pattern::parse("{f}").unwrap();
        pattern.assert_eval(&mut context, "ábčd.JPEG");
    }

    #[test]
    fn eval_complex_input() {
        let mut context = make_context();
        let pattern = Pattern::parse("prefix_{b|a}.{e|u|r'e}").unwrap();
        pattern.assert_eval(&mut context, "prefix_abcd.jpg");
    }

    #[test]
    fn eval_multiple_times() {
        let mut context = make_context();
        let pattern = Pattern::parse("image_{1}_{C|>00}_{c}{E}").unwrap();
        pattern.assert_eval(&mut context, "image_abc_02_1.JPEG");
        pattern.assert_eval(&mut context, "image_abc_03_2.JPEG");
        pattern.assert_eval(&mut context, "image_abc_04_3.JPEG");
        assert_eq!(
            context,
            EvalContext {
                path: Path::new("root/parent/ábčd.JPEG"),
                local_counter: 4,
                global_counter: 5,
                capture_groups: vec!["abc".to_string()],
            }
        )
    }

    #[test]
    fn eval_error() {
        let mut context = make_context();
        let pattern = Pattern::parse("image_{2}.{e}").unwrap();
        pattern.assert_eval_error(
            &mut context,
            EvalError {
                message: "Value exceeded number of regex capture groups",
                variable: &Parsed {
                    value: Variable::CaptureGroup(2),
                    start: 7,
                    end: 8,
                },
            },
        )
    }

    fn assert_parse_items(string: &str, items: Vec<Parsed<PatternItem>>) {
        assert_eq!(Pattern::parse(string), Ok(Pattern { items }));
    }

    fn assert_parse_error(string: &str, error: ParseError) {
        assert_eq!(Pattern::parse(string), Err(error));
    }

    fn make_context<'a>() -> EvalContext<'a> {
        EvalContext {
            path: Path::new("root/parent/ábčd.JPEG"),
            local_counter: 1,
            global_counter: 2,
            capture_groups: vec!["abc".to_string()],
        }
    }

    impl Pattern {
        fn assert_eval(&self, context: &mut EvalContext, result: &str) {
            assert_eq!(self.eval(context), Ok(result.to_string()));
        }

        fn assert_eval_error(&self, context: &mut EvalContext, error: EvalError) {
            assert_eq!(self.eval(context), Err(error));
        }
    }
}
