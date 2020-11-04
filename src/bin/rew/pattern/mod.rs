use crate::pattern::lexer::Lexer;
use crate::pattern::parse::Parsed;
use crate::pattern::parser::Item;
use crate::pattern::parser::Parser;
use crate::pattern::variable::Variable;

mod char;
pub mod eval;
mod explain;
mod filter;
pub mod help;
mod lexer;
mod number;
pub mod parse;
mod parser;
mod range;
mod reader;
mod regex;
mod substitution;
mod symbols;
#[cfg(test)]
mod testing;
mod variable;

#[derive(Debug, PartialEq)]
pub struct Pattern {
    source: String,
    items: Vec<Parsed<Item>>,
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
                                            value: context.path.to_string_lossy().to_string(),
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
                                value: context.path.to_string_lossy().to_string(),
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
    use crate::pattern::parse::Parsed;
    use crate::pattern::range::Range;
    use crate::pattern::substitution::Substitution;
    use crate::pattern::testing::{make_eval_context, make_parsed};
    use crate::utils::AnyString;

    #[test]
    fn parse_default_escape() {
        assert_eq!(
            Pattern::parse("_#{{p|l}#}_", None),
            Ok(Pattern {
                source: String::from("_#{{p|l}#}_"),
                items: vec![
                    Parsed {
                        value: Item::Constant(String::from("_{")),
                        range: 0..3,
                    },
                    Parsed {
                        value: Item::Expression {
                            variable: Parsed {
                                value: Variable::InputPath,
                                range: 4..5,
                            },
                            filters: vec![Parsed {
                                value: Filter::ToLowercase,
                                range: 6..7,
                            }],
                        },
                        range: 3..8,
                    },
                    Parsed {
                        value: Item::Constant(String::from("}_")),
                        range: 8..11,
                    },
                ]
            })
        )
    }

    #[test]
    fn parse_custom_escape() {
        assert_eq!(
            Pattern::parse("_\\{{p|l}\\}_", Some('\\')),
            Ok(Pattern {
                source: String::from("_\\{{p|l}\\}_"),
                items: vec![
                    Parsed {
                        value: Item::Constant(String::from("_{")),
                        range: 0..3,
                    },
                    Parsed {
                        value: Item::Expression {
                            variable: Parsed {
                                value: Variable::InputPath,
                                range: 4..5,
                            },
                            filters: vec![Parsed {
                                value: Filter::ToLowercase,
                                range: 6..7,
                            }],
                        },
                        range: 3..8,
                    },
                    Parsed {
                        value: Item::Constant(String::from("}_")),
                        range: 8..11,
                    },
                ]
            })
        )
    }

    #[test]
    fn parse_error() {
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
            items: vec![
                make_parsed(Item::Constant(String::from("a"))),
                make_parsed(Item::Expression {
                    variable: make_parsed(Variable::FileName),
                    filters: Vec::new(),
                }),
            ],
        };
        assert!(!pattern.uses_local_counter());
        assert!(!pattern.uses_global_counter());
        assert!(!pattern.uses_regex_captures());
    }

    #[test]
    fn uses_local_counter() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![make_parsed(Item::Expression {
                variable: make_parsed(Variable::LocalCounter),
                filters: Vec::new(),
            })],
        };
        assert!(pattern.uses_local_counter());
    }

    #[test]
    fn uses_global_counter() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![make_parsed(Item::Expression {
                variable: make_parsed(Variable::GlobalCounter),
                filters: Vec::new(),
            })],
        };
        assert!(pattern.uses_global_counter());
    }

    #[test]
    fn uses_regex_captures() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![make_parsed(Item::Expression {
                variable: make_parsed(Variable::RegexCapture(1)),
                filters: Vec::new(),
            })],
        };
        assert!(pattern.uses_regex_captures());
    }

    #[test]
    fn eval_constant() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![make_parsed(Item::Constant(String::from("abc")))],
        };
        assert_eq!(pattern.eval(&make_eval_context()), Ok(String::from("abc")));
    }

    #[test]
    fn eval_expression() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![make_parsed(Item::Expression {
                variable: make_parsed(Variable::FileName),
                filters: Vec::new(),
            })],
        };
        assert_eq!(
            pattern.eval(&make_eval_context()),
            Ok(String::from("file.ext"))
        );
    }

    #[test]
    fn eval_expression_single_filter() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![make_parsed(Item::Expression {
                variable: make_parsed(Variable::FileName),
                filters: vec![make_parsed(Filter::ToUppercase)],
            })],
        };
        assert_eq!(
            pattern.eval(&make_eval_context()),
            Ok(String::from("FILE.EXT"))
        );
    }

    #[test]
    fn eval_expression_multiple_filters() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![make_parsed(Item::Expression {
                variable: make_parsed(Variable::FileName),
                filters: vec![
                    make_parsed(Filter::ToUppercase),
                    make_parsed(Filter::Substring(Range::To(4))),
                ],
            })],
        };
        assert_eq!(pattern.eval(&make_eval_context()), Ok(String::from("FILE")));
    }

    #[test]
    fn eval_multiple_constants_and_expressions() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![
                make_parsed(Item::Constant(String::from("prefix_"))),
                make_parsed(Item::Expression {
                    variable: make_parsed(Variable::BaseName),
                    filters: vec![make_parsed(Filter::Substring(Range::To(3)))],
                }),
                make_parsed(Item::Constant(String::from("_"))),
                make_parsed(Item::Expression {
                    variable: make_parsed(Variable::RegexCapture(1)),
                    filters: Vec::new(),
                }),
                make_parsed(Item::Constant(String::from("_"))),
                make_parsed(Item::Expression {
                    variable: make_parsed(Variable::LocalCounter),
                    filters: Vec::new(),
                }),
                make_parsed(Item::Constant(String::from("_"))),
                make_parsed(Item::Expression {
                    variable: make_parsed(Variable::GlobalCounter),
                    filters: Vec::new(),
                }),
                make_parsed(Item::Constant(String::from("."))),
                make_parsed(Item::Expression {
                    variable: make_parsed(Variable::Extension),
                    filters: vec![
                        make_parsed(Filter::ToUppercase),
                        make_parsed(Filter::ReplaceAll(Substitution {
                            value: String::from("X"),
                            replacement: String::from(""),
                        })),
                    ],
                }),
            ],
        };
        assert_eq!(
            pattern.eval(&make_eval_context()),
            Ok(String::from("prefix_fil_abc_1_2.ET"))
        );
    }

    #[test]
    fn eval_variable_error() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![make_parsed(Item::Expression {
                variable: Parsed {
                    value: Variable::CanonicalPath,
                    range: 1..2,
                },
                filters: Vec::new(),
            })],
        };
        assert_eq!(
            pattern.eval(&make_eval_context()),
            Err(eval::Error {
                kind: eval::ErrorKind::CanonicalizationFailed(AnyString(String::from(
                    "This string is not compared by assertion"
                ))),
                cause: eval::ErrorCause::Variable(&Variable::CanonicalPath),
                value: String::from("root/parent/file.ext"),
                range: &(1..2usize),
            })
        );
    }
}
