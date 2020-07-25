use crate::pattern::lexer::Lexer;
use crate::pattern::parse::Output;
use crate::pattern::parser::Item;
use crate::pattern::parser::Parser;
use crate::pattern::variable::Variable;

mod char;
pub mod eval;
mod explain;
mod filter;
mod lexer;
mod number;
pub mod parse;
mod parser;
mod range;
mod reader;
mod regex;
mod substitution;
mod symbols;
mod variable;

#[derive(Debug, PartialEq)]
pub struct Pattern {
    source: String,
    items: Vec<Output<Item>>,
}

impl Pattern {
    pub fn parse(source: &str, escape: Option<char>) -> parse::Result<Self> {
        let mut lexer = Lexer::new(source);

        if let Some(escape) = escape {
            lexer.set_escape(escape);
        }

        let mut parser = Parser::new(lexer);
        let items = parser.parse_items()?;

        Ok(Self {
            source: String::from(source),
            items,
        })
    }

    pub fn uses_local_counter(&self) -> bool {
        self.uses_variable(|variable| *variable == Variable::LocalCounter)
    }

    pub fn uses_global_counter(&self) -> bool {
        self.uses_variable(|variable| *variable == Variable::GlobalCounter)
    }

    pub fn uses_regex_captures(&self) -> bool {
        self.uses_variable(|variable| matches!(variable, Variable::RegexCapture(_)))
    }

    fn uses_variable<F: Fn(&Variable) -> bool>(&self, test: F) -> bool {
        self.items.iter().any(|item| {
            if let Item::Expression { variable, .. } = &item.value {
                test(&variable.value)
            } else {
                false
            }
        })
    }

    pub fn eval(&self, context: &eval::Context) -> eval::Result<String> {
        let mut output = String::new();

        for item in &self.items {
            match &item.value {
                Item::Constant(string) => output.push_str(string),
                Item::Expression { variable, filters } => {
                    match variable.value.eval(context) {
                        Ok(mut string) => {
                            for filter in filters.iter() {
                                match filter.value.eval(string) {
                                    Ok(result) => string = result,
                                    Err(kind) => {
                                        return Err(eval::Error {
                                            kind,
                                            cause: eval::ErrorCause::Filter(&filter.value),
                                            range: &filter.range,
                                        });
                                    }
                                }
                            }
                            output.push_str(&string)
                        }
                        Err(kind) => {
                            return Err(eval::Error {
                                kind,
                                cause: eval::ErrorCause::Variable(&variable.value),
                                range: &variable.range,
                            });
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
    use crate::pattern::filter::Filter;
    use crate::pattern::parse::Output;
    use crate::pattern::range::Range;
    use crate::pattern::substitution::Substitution;
    use ::regex::Regex;
    use std::path::Path;

    #[test]
    fn parses_with_default_escape() {
        assert_eq!(
            Pattern::parse("_#{{p|l}#}_", None),
            Ok(Pattern {
                source: String::from("_#{{p|l}#}_"),
                items: vec![
                    Output {
                        value: Item::Constant(String::from("_{")),
                        range: 0..3,
                    },
                    Output {
                        value: Item::Expression {
                            variable: Output {
                                value: Variable::Path,
                                range: 4..5,
                            },
                            filters: vec![Output {
                                value: Filter::ToLowercase,
                                range: 6..7,
                            }],
                        },
                        range: 3..8,
                    },
                    Output {
                        value: Item::Constant(String::from("}_")),
                        range: 8..11,
                    },
                ]
            })
        )
    }

    #[test]
    fn parses_with_custom_escape() {
        assert_eq!(
            Pattern::parse("_\\{{p|l}\\}_", Some('\\')),
            Ok(Pattern {
                source: String::from("_\\{{p|l}\\}_"),
                items: vec![
                    Output {
                        value: Item::Constant(String::from("_{")),
                        range: 0..3,
                    },
                    Output {
                        value: Item::Expression {
                            variable: Output {
                                value: Variable::Path,
                                range: 4..5,
                            },
                            filters: vec![Output {
                                value: Filter::ToLowercase,
                                range: 6..7,
                            }],
                        },
                        range: 3..8,
                    },
                    Output {
                        value: Item::Constant(String::from("}_")),
                        range: 8..11,
                    },
                ]
            })
        )
    }

    #[test]
    fn parses_with_error() {
        assert_eq!(
            Pattern::parse("{", None),
            Err(parse::Error {
                kind: parse::ErrorKind::ExpectedVariable,
                range: 1..1
            })
        )
    }

    #[test]
    fn uses_none() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![output(Item::Expression {
                variable: output(Variable::FileName),
                filters: Vec::new(),
            })],
        };
        assert_eq!(pattern.uses_local_counter(), false);
        assert_eq!(pattern.uses_global_counter(), false);
        assert_eq!(pattern.uses_regex_captures(), false);
    }

    #[test]
    fn uses_local_counter() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![output(Item::Expression {
                variable: output(Variable::LocalCounter),
                filters: Vec::new(),
            })],
        };
        assert_eq!(pattern.uses_local_counter(), true);
    }

    #[test]
    fn uses_global_counter() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![output(Item::Expression {
                variable: output(Variable::GlobalCounter),
                filters: Vec::new(),
            })],
        };
        assert_eq!(pattern.uses_global_counter(), true);
    }

    #[test]
    fn uses_regex_captures() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![output(Item::Expression {
                variable: output(Variable::RegexCapture(1)),
                filters: Vec::new(),
            })],
        };
        assert_eq!(pattern.uses_regex_captures(), true);
    }

    #[test]
    fn evals_constant() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![output(Item::Constant(String::from("abc")))],
        };
        assert_eq!(pattern.eval(&make_context()), Ok(String::from("abc")));
    }

    #[test]
    fn evals_expression() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![output(Item::Expression {
                variable: output(Variable::FileName),
                filters: Vec::new(),
            })],
        };
        assert_eq!(pattern.eval(&make_context()), Ok(String::from("file.ext")));
    }

    #[test]
    fn evals_expression_single_filter() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![output(Item::Expression {
                variable: output(Variable::FileName),
                filters: vec![output(Filter::ToUppercase)],
            })],
        };
        assert_eq!(pattern.eval(&make_context()), Ok(String::from("FILE.EXT")));
    }

    #[test]
    fn evals_expression_multiple_filters() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![output(Item::Expression {
                variable: output(Variable::FileName),
                filters: vec![
                    output(Filter::ToUppercase),
                    output(Filter::Substring(Range::To(4))),
                ],
            })],
        };
        assert_eq!(pattern.eval(&make_context()), Ok(String::from("FILE")));
    }

    #[test]
    fn evals_multiple_constants_and_expressions() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![
                output(Item::Constant(String::from("prefix_"))),
                output(Item::Expression {
                    variable: output(Variable::BaseName),
                    filters: vec![output(Filter::Substring(Range::To(3)))],
                }),
                output(Item::Constant(String::from("_"))),
                output(Item::Expression {
                    variable: output(Variable::RegexCapture(1)),
                    filters: Vec::new(),
                }),
                output(Item::Constant(String::from("_"))),
                output(Item::Expression {
                    variable: output(Variable::LocalCounter),
                    filters: Vec::new(),
                }),
                output(Item::Constant(String::from("_"))),
                output(Item::Expression {
                    variable: output(Variable::GlobalCounter),
                    filters: Vec::new(),
                }),
                output(Item::Constant(String::from("."))),
                output(Item::Expression {
                    variable: output(Variable::Extension),
                    filters: vec![
                        output(Filter::ToUppercase),
                        output(Filter::ReplaceAll(Substitution {
                            value: String::from("X"),
                            replacement: String::from(""),
                        })),
                    ],
                }),
            ],
        };
        assert_eq!(
            pattern.eval(&make_context()),
            Ok(String::from("prefix_fil_abc_1_2.ET"))
        );
    }

    fn output<T>(value: T) -> Output<T> {
        Output { value, range: 0..0 }
    }

    fn make_context<'a>() -> eval::Context<'a> {
        eval::Context {
            path: Path::new("root/parent/file.ext"),
            local_counter: 1,
            global_counter: 2,
            regex_captures: Regex::new("(.*)").unwrap().captures("abc"),
        }
    }
}
